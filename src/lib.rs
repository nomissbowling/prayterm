#![doc(html_root_url = "https://docs.rs/prayterm/0.1.0")]
//! prayterm realtime play nonblocking terminal for Rust with crossterm
//!

use std::fmt;
use std::error::Error;
use std::io::{stdout, Write};
use std::time;
use std::thread;
use std::sync::mpsc;

use crossterm::{execute, queue};
use crossterm::terminal::{self, disable_raw_mode, enable_raw_mode};
use crossterm::cursor;
use crossterm::style::{self, Attribute};
use crossterm::event::{self, Event};

/// NopColor
pub trait NopColor {
  /// nop
  fn nop(&self) -> style::Color;
}

/// NopColor for style::Color
impl NopColor for style::Color {
  /// nop
  fn nop(&self) -> style::Color { *self }
}

/// Rgb
#[derive(Debug, Clone)]
pub struct Rgb(pub u8, pub u8, pub u8);

/// NopColor for Rgb
impl NopColor for Rgb {
  /// nop
  fn nop(&self) -> style::Color {
    style::Color::Rgb{r: self.0, g: self.1, b: self.2}
  }
}

/// PrayTerm
// #[derive(Debug)]
pub struct PrayTerm {
  /// kind
  pub k: u16,
  /// width
  pub w: u16,
  /// height
  pub h: u16,
  /// so stdout
  pub so: Box<dyn Write>
}

/// Debug
impl fmt::Debug for PrayTerm {
  /// fmt
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "({}, {}) [stdout]", self.w, self.h)
  }
}

/// Display
impl fmt::Display for PrayTerm {
  /// fmt
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{:?}", self)
  }
}

/// PrayTerm
impl PrayTerm {
  /// constructor
  pub fn new(k: u16) -> Result<Self, Box<dyn Error>> {
    let (w, h) = terminal::size()?;
    enable_raw_mode()?;
    let mut so = stdout();
    if k & 5 != 0 { execute!(so, terminal::EnterAlternateScreen)?; }
    if k & 6 != 0 { execute!(so, event::EnableMouseCapture)?; }
    Ok(PrayTerm{k, w, h, so: Box::new(so)})
  }

  /// begin
  pub fn begin(&mut self) -> Result<(), Box<dyn Error>> {
    execute!(self.so,
      cursor::SetCursorStyle::DefaultUserShape, // Blinking... Steady...
      cursor::Hide,
      terminal::Clear(terminal::ClearType::All))?;
    Ok(())
  }

  /// fin
  pub fn fin(&mut self) -> Result<(), Box<dyn Error>> {
    execute!(self.so,
      cursor::SetCursorStyle::BlinkingUnderScore, // Block[] UnderScore_ Bar|
      cursor::Show)?;
    if self.k & 6 != 0 { execute!(self.so, event::DisableMouseCapture)?; }
    if self.k & 5 != 0 { execute!(self.so, terminal::LeaveAlternateScreen)?; }
    disable_raw_mode()?;
    Ok(())
  }

  /// style
  pub fn style(&mut self, s: Attribute) -> Result<(), Box<dyn Error>> {
    queue!(self.so, style::SetAttribute(s))?;
    Ok(())
  }

  /// write
  pub fn wr(&mut self, x: u16, y: u16,
    st: u16, bg: impl NopColor, fg: impl NopColor, msg: &String) ->
    Result<(), Box<dyn Error>> {
    let styles: Vec<Attribute> = vec![Attribute::Bold, Attribute::Italic];
    for (i, s) in styles.iter().enumerate() {
      if st & 2^(i as u16) != 0 { self.style(*s)?; }
    }
    queue!(self.so,
      cursor::MoveTo(x, y),
      style::SetBackgroundColor(bg.nop()), style::SetForegroundColor(fg.nop()),
      style::Print(msg), style::ResetColor)?;
    self.so.flush()?;
    Ok(())
  }

  /// prepare thread
  pub fn prepare_thread(&self, ms: time::Duration) ->
    Result<(mpsc::Sender<Event>, mpsc::Receiver<Event>), Box<dyn Error>> {
    let (tx, rx) = mpsc::channel();
    if true { // closure once
      let tx = tx.clone();
      let _handle = thread::spawn(move || { // for non blocking to fetch event
        loop { // loop forever
          if !event::poll(ms).expect("poll") { () } // non blocking
          else {
            match event::read().expect("read") { // blocking
            ev => {
              tx.send(ev).expect("send");
            }
            }
          }
          ()
        }
        // () // not be arrived here (will not be disconnected)
      });
    }
    Ok((tx, rx))
  }
}

/// test with [-- --nocapture] or [-- --show-output]
#[cfg(test)]
mod tests {
  use super::{PrayTerm, Rgb};
  use crossterm::style::Color;

  /// test a
  #[test]
  fn test_a() {
    let s = String::from_utf8("ABC".into()).expect("utf8");
    let mut tm = PrayTerm::new(2).expect("construct");
    tm.begin().expect("begin");
    tm.wr(0, 48, 3, Color::Blue, Color::Yellow, &s).expect("wr");
    tm.wr(0, 49, 3, Rgb(240, 192, 32), Rgb(240, 32, 192), &s).expect("wr");
    tm.fin().expect("fin");
    assert_eq!(tm.w, 80);
    assert_eq!(tm.h, 50);
  }
}
