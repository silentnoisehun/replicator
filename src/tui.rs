use crate::spine::Spine;
use crate::corn_kernel::CornKernel;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap, List, ListItem},
    Terminal,
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io;
use std::time::{Duration, Instant};

pub struct TuiApp {
    input: String,
    messages: Vec<String>,
    spine: Spine,
    last_seq: u64,
}

impl TuiApp {
    pub fn new() -> io::Result<Self> {
        let spine_id: [u8; 16] = *b"HOPE-TUI-ZONE-01";
        let spine = Spine::open_default(spine_id)?;
        let last_seq = spine.writer_seq();
        
        Ok(Self {
            input: String::new(),
            messages: vec!["[SYSTEM] Welcome to HOPE-OS TUI. Connection established.".to_string()],
            spine,
            last_seq,
        })
    }

    pub fn run(&mut self) -> io::Result<()> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let tick_rate = Duration::from_millis(100);
        let mut last_tick = Instant::now();

        loop {
            terminal.draw(|f| self.ui(f))?;

            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Enter => {
                            let msg = self.input.drain(..).collect::<String>();
                            if !msg.is_empty() {
                                let mut kernel = CornKernel::empty();
                                for (i, chunk) in msg.as_bytes().chunks(32).enumerate().take(8) {
                                    kernel.write_layer(i, chunk);
                                }
                                self.spine.write(&kernel);
                                self.messages.push(format!("[YOU] {}", msg));
                            }
                        }
                        KeyCode::Char(c) => {
                            self.input.push(c);
                        }
                        KeyCode::Backspace => {
                            self.input.pop();
                        }
                        KeyCode::Esc => {
                            break;
                        }
                        _ => {}
                    }
                }
            }

            if last_tick.elapsed() >= tick_rate {
                self.update_spine();
                last_tick = Instant::now();
            }
        }

        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        Ok(())
    }

    fn update_spine(&mut self) {
        let current_w = self.spine.writer_seq();
        while self.last_seq < current_w {
            if let Some(kernel) = self.spine.read(self.last_seq) {
                let data = kernel.deep_read();
                let msg = std::str::from_utf8(data)
                    .unwrap_or("")
                    .trim_matches(char::from(0))
                    .trim();
                
                if !msg.is_empty() {
                    self.messages.push(msg.to_string());
                }
            }
            self.last_seq += 1;
        }
    }

    fn ui(&self, f: &mut ratatui::Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(1),
                Constraint::Length(3),
            ])
            .split(f.size());

        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(70),
                Constraint::Percentage(30),
            ])
            .split(chunks[0]);

        // Chat Box
        let items: Vec<ListItem> = self.messages
            .iter()
            .rev()
            .map(|m| {
                let style = if m.starts_with("[YOU]") {
                    Style::default().fg(Color::Cyan)
                } else if m.starts_with("AGENT:") {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(Color::Yellow)
                };
                ListItem::new(Line::from(Span::styled(m, style)))
            })
            .collect();

        let chat = List::new(items)
            .block(Block::default().borders(Borders::ALL).title(" RONGYASZ CHAT "))
            .start_corner(ratatui::layout::Corner::BottomLeft);
        f.render_widget(chat, main_chunks[0]);

        // VM Screen / Visualizer
        let mut vm_content = String::new();
        vm_content.push_str("── HOPE-VM STATUS ──\n");
        vm_content.push_str(&format!("TICK: {:08}\n", self.last_seq));
        vm_content.push_str("BUS: ACTIVE\n\n");
        vm_content.push_str("── VRAM MATRIX ──\n");
        
        // Egyszerű Braille-szerű vizualizáció a Spine adataiból
        for i in 0..10 {
            for j in 0..15 {
                let idx = (i * 15 + j) % 32;
                if self.last_seq % (idx as u64 + 1) == 0 {
                    vm_content.push('⣿');
                } else {
                    vm_content.push('░');
                }
            }
            vm_content.push('\n');
        }

        let vm_screen = Paragraph::new(vm_content)
            .block(Block::default().borders(Borders::ALL).title(" VM SCREEN "))
            .style(Style::default().fg(Color::Magenta));
        f.render_widget(vm_screen, main_chunks[1]);

        // Input Box
        let input = Paragraph::new(self.input.as_str())
            .style(Style::default().fg(Color::White))
            .block(Block::default().borders(Borders::ALL).title(" COMMAND INJECTION "));
        f.render_widget(input, chunks[1]);
    }
}
