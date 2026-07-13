use owo_colors::OwoColorize;

pub fn header(text: &str) {
    println!("\n{}", text.bold().cyan());
    println!("{}", "─".repeat(text.len()).cyan());
}

pub fn step(text: &str) {
    println!("{} {}", "→".blue().bold(), text.dimmed());
}

pub fn success(text: &str) {
    println!("{} {}", "✓".green().bold(), text.green());
}

pub fn failure(text: &str) {
    println!("{} {}", "✗".red().bold(), text.red());
}

pub fn info(text: &str) {
    println!("{} {}", "ℹ".yellow().bold(), text);
}

pub fn field(label: &str, value: &str) {
    println!("  {:<24} {}", label.dimmed(), value);
}

pub fn output(text: &str) {
    println!("  {}", text);
}
