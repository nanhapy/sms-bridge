/**
 * UDP 自动发现局域网内的电脑接收端（三通道，兼容性强）
 * 通道 1（最可靠）：组播发送到 239.255.42.99:port —— 电脑端加入该组播组接收。
 *        参照 LocalSend 鸿蒙版（localsend-ohos）真机实测：NEXT 上组播可用且比广播可靠；
 *        发送方无需加入组播组，普通 UDP send 即可。
 * 通道 2：255.255.255.255 全网广播
 * 通道 3：根据本机网卡信息计算出的子网定向广播（如 192.168.3.255），
 *        规避部分机型对全网广播的限制
 * 电脑端收到后回复：SMSBRIDGE_READY|主机名|端口
 */
import { socket, connection } from '@kit.NetworkKit';
import { util } from '@kit.ArkTS';

const MAGIC_DISCOVER = 'SMSBRIDGE_DISCOVER';
const MAGIC_READY = 'SMSBRIDGE_READY';
const MULTICAST_GROUP = '239.255.42.99';

export function discoverPc(port: number, timeoutMs: number,
  onFound: (ip: string, host: string, portStr: string) => void,
  onDone: () => void): void {
  const udp = socket.constructUDPSocketInstance();
  let finished = false;

  const finish = (): void => {
    if (finished) {
      return;
    }
    finished = true;
    try {
      udp.close();
    } catch (e) {
      // 忽略关闭异常
    }
    onDone();
  };

  udp.bind({ address: '0.0.0.0', port: 0 }).then(() => {
    return udp.setExtraOptions({ broadcast: true, receiveBufferSize: 4096 });
  }).then(() => {
    udp.on('message', (info: socket.SocketMessageInfo) => {
      const text = decode(info.message);
      if (text.startsWith(MAGIC_READY + '|')) {
        const parts = text.split('|');
        const host = parts.length > 1 ? parts[1] : 'PC';
        const portStr = parts.length > 2 ? parts[2] : String(port);
        // 优先用包内携带的「与手机同网段」IP（多网卡/VPN 电脑可靠），否则用回复包来源 IP
        const payloadIp = parts.length > 3 ? parts[3] : '';
        const ip = isValidIpv4(payloadIp) ? payloadIp : info.remoteInfo.address;
        onFound(ip, host, portStr);
      }
    });
    return getBroadcastTargets();
  }).then((targets: string[]) => {
    const sendAll = async (): Promise<void> => {
      for (const t of targets) {
        try {
          await udp.send({ data: MAGIC_DISCOVER, address: { address: t, family: 1, port: port } });
        } catch (e) {
          // 该目标不可达，尝试下一个
        }
      }
    };
    return sendAll();
  }).catch(() => {
    finish();
  });

  setTimeout(finish, timeoutMs);
}

async function getBroadcastTargets(): Promise<string[]> {
  const targets: string[] = [MULTICAST_GROUP, '255.255.255.255'];
  try {
    const net = await connection.getDefaultNet();
    const props = await connection.getConnectionProperties(net);
    const links = props.linkAddresses;
    if (links) {
      for (const link of links) {
        const addr = link.address.address;
        const family = link.address.family;
        if (family !== 1 || !addr) {
          continue;
        }
        if (addr.startsWith('127.') || addr.startsWith('169.254')) {
          continue;
        }
        const prefix = link.prefixLength ?? 24;
        const bc = subnetBroadcast(addr, prefix);
        if (bc && targets.indexOf(bc) < 0) {
          targets.push(bc);
        }
      }
    }
  } catch (e) {
    // 获取网卡信息失败，仅用全网广播
  }
  return targets;
}

function subnetBroadcast(ip: string, prefix: number): string | null {
  const parts = ip.split('.');
  if (parts.length !== 4) {
    return null;
  }
  const nums: number[] = [];
  for (const p of parts) {
    const n = Number(p);
    if (isNaN(n) || n < 0 || n > 255) {
      return null;
    }
    nums.push(n);
  }
  const safePrefix = prefix <= 0 ? 0 : (prefix >= 32 ? 32 : prefix);
  const mask = safePrefix === 0 ? 0 : (0xFFFFFFFF - (Math.pow(2, 32 - safePrefix) - 1));
  const ipNum = nums[0] * 16777216 + nums[1] * 65536 + nums[2] * 256 + nums[3];
  const bcNum = ipNum | mask;
  const out: number[] = [];
  out.push(Math.floor(bcNum / 16777216) % 256);
  out.push(Math.floor(bcNum / 65536) % 256);
  out.push(Math.floor(bcNum / 256) % 256);
  out.push(bcNum % 256);
  return `${out[0]}.${out[1]}.${out[2]}.${out[3]}`;
}

function isValidIpv4(ip: string): boolean {
  if (!ip || ip.length === 0) {
    return false;
  }
  const parts = ip.split('.');
  if (parts.length !== 4) {
    return false;
  }
  for (const p of parts) {
    const n = Number(p);
    if (isNaN(n) || n < 0 || n > 255) {
      return false;
    }
  }
  return true;
}

function decode(buf: ArrayBuffer): string {
  const decoder = util.TextDecoder.create('utf-8');
  return decoder.decodeToString(new Uint8Array(buf));
}
