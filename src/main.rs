use std::env;

use rustshark::{capture, tui};

enum CaptureMode {
    File(String),
    Interface(String),
}

fn parse_args() -> Option<CaptureMode> {
    let args: Vec<String> = env::args().collect();
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-r" if i + 1 < args.len() => return Some(CaptureMode::File(args[i + 1].clone())),
            "-i" if i + 1 < args.len() => return Some(CaptureMode::Interface(args[i + 1].clone())),
            _ => {}
        }
        i += 1;
    }
    None
}

fn main() {
    let mode = parse_args().unwrap_or_else(|| {
        eprintln!("사용법: rustshark -r <pcap파일> | -i <인터페이스>");
        std::process::exit(1);
    });

    let (tx, rx) = crossbeam_channel::bounded(1024);

    std::thread::spawn(move || match mode {
        CaptureMode::File(path)       => capture::read_pcap_file(&path, &tx),
        CaptureMode::Interface(iface) => capture::start_capture(&iface, &tx),
    });

    if let Err(e) = tui::App::new().run(rx) {
        eprintln!("오류: {e}");
        std::process::exit(1);
    }
}
