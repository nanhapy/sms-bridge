export interface HistoryRecord {
  receivedAt: number;
  text: string;
  code: string | null;
  source: string;
}

export type ReceiverStatus =
  | { kind: "starting" }
  | { kind: "listening"; port: number }
  | { kind: "degraded"; port: number; message: string }
  | { kind: "unavailable"; port: number; message: string };

export interface AppSnapshot {
  history: HistoryRecord[];
  receiverStatus: ReceiverStatus;
  autostartEnabled: boolean;
  storageWarning: string | null;
}
