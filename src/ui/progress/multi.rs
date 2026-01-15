// Wed Jan 15 2026 - Alex

use indicatif::{MultiProgress as IndicatifMulti, ProgressBar, ProgressStyle, ProgressDrawTarget};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct MultiProgress {
    multi: IndicatifMulti,
    bars: Arc<Mutex<HashMap<String, ProgressBar>>>,
    default_style: ProgressStyle,
}

impl MultiProgress {
    pub fn new() -> Self {
        let default_style = ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("█▓▒░ ");

        Self {
            multi: IndicatifMulti::new(),
            bars: Arc::new(Mutex::new(HashMap::new())),
            default_style,
        }
    }

    pub fn add_bar(&self, name: &str, total: u64, message: &str) -> ProgressBar {
        let pb = ProgressBar::new(total);
        pb.set_style(self.default_style.clone());
        pb.set_message(message.to_string());

        let pb = self.multi.add(pb);

        let mut bars = self.bars.lock().unwrap();
        bars.insert(name.to_string(), pb.clone());

        pb
    }

    pub fn add_bar_with_style(&self, name: &str, total: u64, message: &str, template: &str) -> ProgressBar {
        let style = ProgressStyle::default_bar()
            .template(template)
            .unwrap()
            .progress_chars("█▓▒░ ");

        let pb = ProgressBar::new(total);
        pb.set_style(style);
        pb.set_message(message.to_string());

        let pb = self.multi.add(pb);

        let mut bars = self.bars.lock().unwrap();
        bars.insert(name.to_string(), pb.clone());

        pb
    }

    pub fn get_bar(&self, name: &str) -> Option<ProgressBar> {
        let bars = self.bars.lock().unwrap();
        bars.get(name).cloned()
    }

    pub fn remove_bar(&self, name: &str) {
        let mut bars = self.bars.lock().unwrap();
        if let Some(pb) = bars.remove(name) {
            pb.finish_and_clear();
        }
    }

    pub fn clear(&self) {
        let mut bars = self.bars.lock().unwrap();
        for (_, pb) in bars.drain() {
            pb.finish_and_clear();
        }
    }

    pub fn finish_all(&self) {
        let bars = self.bars.lock().unwrap();
        for pb in bars.values() {
            pb.finish();
        }
    }

    pub fn set_hidden(&self) {
        self.multi.set_draw_target(ProgressDrawTarget::hidden());
    }

    pub fn set_visible(&self) {
        self.multi.set_draw_target(ProgressDrawTarget::stderr());
    }

    pub fn inner(&self) -> &IndicatifMulti {
        &self.multi
    }

    pub fn join(&self) -> std::io::Result<()> {
        self.multi.clear()
    }
}

impl Default for MultiProgress {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for MultiProgress {
    fn clone(&self) -> Self {
        Self {
            multi: IndicatifMulti::new(),
            bars: self.bars.clone(),
            default_style: self.default_style.clone(),
        }
    }
}
