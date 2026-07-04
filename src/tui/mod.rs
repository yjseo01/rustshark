pub mod hex_dump;
pub mod layout;
pub mod packet_detail;
pub mod packet_list;

use std::io::Stdout;
use std::time::Duration;

use crossbeam_channel::Receiver;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};
use ratatui::widgets::TableState;

use crate::capture;
use crate::filter::DisplayFilter;
use crate::types::ParsedPacket;

enum AppMode {
    Normal,
    Filter,
}

pub struct App {
    packets: Vec<ParsedPacket>,
    filtered: Vec<usize>,
    table_state: TableState,
    filter: DisplayFilter,
    filter_input: String,
    mode: AppMode,
    paused: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            packets: Vec::new(),
            filtered: Vec::new(),
            table_state: TableState::default(),
            filter: DisplayFilter::default(),
            filter_input: String::new(),
            mode: AppMode::Normal,
            paused: false,
        }
    }

    pub fn run(mut self, rx: Receiver<ParsedPacket>) -> Result<(), Box<dyn std::error::Error>> {
        enable_raw_mode()?;
        let mut stdout = std::io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let result = self.event_loop(&mut terminal, rx);

        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        terminal.show_cursor()?;

        result
    }

    fn event_loop(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
        rx: Receiver<ParsedPacket>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            // 채널에서 패킷 드레인
            if !self.paused {
                while let Ok(pkt) = rx.try_recv() {
                    self.push_packet(pkt);
                }
            }

            // 렌더링
            terminal.draw(|frame| self.render(frame))?;

            // 이벤트 처리 (50ms 타임아웃)
            if event::poll(Duration::from_millis(50))? {
                if let Event::Key(key) = event::read()? {
                    if self.handle_key(key) {
                        break;
                    }
                }
            }
        }
        Ok(())
    }

    fn render(&mut self, frame: &mut Frame) {
        let layout = layout::split(frame.area());

        // 필터 바
        let (filter_text, filter_style) = match self.mode {
            AppMode::Filter => (
                format!("Filter: {}_", self.filter_input),
                Style::default().fg(Color::Yellow),
            ),
            AppMode::Normal => {
                let s = self.filter.display_str();
                let text = if s.is_empty() {
                    "[/] 필터  [j/k↑↓] 이동  [p] 일시정지  [s] 저장  [q] 종료".to_string()
                } else {
                    format!("Filter: {s}  [Esc 클리어]")
                };
                (text, Style::default().fg(Color::DarkGray))
            }
        };

        let filter_bar = Paragraph::new(filter_text)
            .style(filter_style)
            .block(Block::default().title("RustShark").borders(Borders::ALL));
        frame.render_widget(filter_bar, layout.filter_bar);

        // 패킷 목록
        packet_list::render(
            frame,
            layout.packet_list,
            &self.packets,
            &self.filtered,
            &mut self.table_state,
        );

        // 선택된 패킷 상세 정보
        let selected_packet = self
            .table_state
            .selected()
            .and_then(|i| self.filtered.get(i))
            .and_then(|&i| self.packets.get(i));
        packet_detail::render(frame, layout.packet_detail, selected_packet);

        // Hex Dump
        let raw: &[u8] = selected_packet
            .map(|p| p.raw.as_slice())
            .unwrap_or(&[]);
        hex_dump::render(frame, layout.hex_dump, raw);
    }

    fn push_packet(&mut self, pkt: ParsedPacket) {
        let idx = self.packets.len();
        if self.filter.matches(&pkt) {
            self.filtered.push(idx);
            // 마지막 패킷 선택 유지 (자동 스크롤)
            let n = self.filtered.len();
            let sel = self.table_state.selected();
            if sel.is_none() || sel == Some(n - 2) {
                self.table_state.select(Some(n - 1));
            }
        }
        self.packets.push(pkt);
    }

    fn apply_filter(&mut self) {
        self.filtered = (0..self.packets.len())
            .filter(|&i| self.filter.matches(&self.packets[i]))
            .collect();
        if self.filtered.is_empty() {
            self.table_state.select(None);
        } else {
            self.table_state.select(Some(0));
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        match self.mode {
            AppMode::Normal => self.handle_normal_key(key),
            AppMode::Filter => self.handle_filter_key(key),
        }
    }

    fn handle_normal_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') | KeyCode::Char('Q') => return true,
            KeyCode::Char('p') => self.paused = !self.paused,
            KeyCode::Char('/') => {
                self.filter_input = self.filter.display_str();
                self.mode = AppMode::Filter;
            }
            KeyCode::Esc => {
                // 필터 초기화
                self.filter = DisplayFilter::default();
                self.apply_filter();
            }
            KeyCode::Char('s') => {
                let _ = capture::save_pcap_file("capture.pcap", &self.packets);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let n = self.filtered.len();
                if n > 0 {
                    let sel = self.table_state.selected().unwrap_or(0);
                    self.table_state.select(Some((sel + 1).min(n - 1)));
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                let sel = self.table_state.selected().unwrap_or(0);
                self.table_state.select(Some(sel.saturating_sub(1)));
            }
            KeyCode::Char('g') => {
                if !self.filtered.is_empty() {
                    self.table_state.select(Some(0));
                }
            }
            KeyCode::Char('G') => {
                let n = self.filtered.len();
                if n > 0 {
                    self.table_state.select(Some(n - 1));
                }
            }
            _ => {}
        }
        false
    }

    fn handle_filter_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Enter => {
                self.filter = DisplayFilter::parse(&self.filter_input);
                self.apply_filter();
                self.mode = AppMode::Normal;
            }
            KeyCode::Esc => {
                self.mode = AppMode::Normal;
                self.filter_input.clear();
            }
            KeyCode::Backspace => {
                self.filter_input.pop();
            }
            KeyCode::Char(c) => {
                self.filter_input.push(c);
            }
            _ => {}
        }
        false
    }
}
