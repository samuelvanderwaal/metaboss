use indicatif::{ProgressBar, ProgressStyle};

pub fn create_spinner(msg: &'static str) -> ProgressBar {
    let spinner = ProgressBar::new_spinner();
    spinner.enable_steady_tick(10);
    spinner.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&["▹▹▹▹▹", "▸▹▹▹▹", "▹▸▹▹▹", "▹▹▸▹▹", "▹▹▹▸▹", "▹▹▹▹▸", ""])
            .template("{spinner:.blue}{msg}"),
    );
    spinner.set_message(msg);
    spinner
}

pub fn create_alt_spinner(msg: &'static str) -> ProgressBar {
    let spinner = ProgressBar::new_spinner();
    spinner.enable_steady_tick(80);
    spinner.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&[
                "[    ]", "[=   ]", "[==  ]", "[=== ]", "[ ===]", "[  ==]", "[   =]", "[    ]",
                "[   =]", "[  ==]", "[ ===]", "[====]", "[=== ]", "[==  ]", "[=   ]",
            ])
            .template("{spinner:.blue} {msg}"),
    );
    spinner.set_message(msg);
    spinner
}

pub fn create_progress_bar(msg: &'static str, len: u64) -> ProgressBar {
    let bar = ProgressBar::new(len);
    bar.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.blue} {msg} {wide_bar:.cyan/blue} {pos:>7}/{len:7} {eta_precise}"),
    );
    bar.set_message(msg);
    bar
}
