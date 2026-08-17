#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    sms_bridge_receiver_lib::run();
}
