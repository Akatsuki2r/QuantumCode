//! Shell completion generation
//!
//! Generates shell completions for bash, zsh, fish, and other shells.

use clap_complete::{Shell, generate};
use clap::CommandFactory;
use color_eyre::eyre::Result;
use crate::Cli;

/// Run completion generation
pub async fn run(shell: Option<String>) -> Result<()> {
    match shell {
        Some(s) => {
            let shell_lower = s.to_lowercase();
            match shell_lower.as_str() {
                "bash" => generate_shell(Shell::Bash),
                "zsh" => generate_shell(Shell::Zsh),
                "fish" => generate_shell(Shell::Fish),
                "powershell" => generate_shell(Shell::PowerShell),
                "elvish" => generate_shell(Shell::Elvish),
                _ => {
                    eprintln!("Unknown shell: {}. Use: bash, zsh, fish, powershell, elvish", s);
                    eprintln!("\nTo install completions:");
                    eprintln!("  Bash:  quantumn completions bash >> ~/.bashrc");
                    eprintln!("  Zsh:   quantumn completions zsh > ~/.zsh/completions/_quantumn");
                    eprintln!("  Fish:  quantumn completions fish > ~/.config/fish/completions/quantumn.fish");
                    Ok(())
                }
            }
        }
        None => {
            println!("Usage: quantumn completions <shell>");
            println!("\nGenerate shell completions for quantumn command.");
            println!("\nShells supported:");
            println!("  bash        - Bash completions");
            println!("  zsh         - Zsh completions");
            println!("  fish        - Fish completions");
            println!("  powershell  - PowerShell completions");
            println!("  elvish      - Elvish completions");
            println!("\nExamples:");
            println!("  quantumn completions bash >> ~/.bashrc");
            println!("  quantumn completions zsh > ~/.zsh/completions/_quantumn");
            println!("  quantumn completions fish > ~/.config/fish/completions/quantumn.fish");
            Ok(())
        }
    }
}

/// Generate completions for a specific shell
fn generate_shell(shell: Shell) -> Result<()> {
    let mut cmd = Cli::command();
    let name = cmd.get_name().to_string();

    generate(shell, &mut cmd, name, &mut std::io::stdout());

    println!("\n# Add the above to your shell config to enable completions.");
    println!("# Then restart your shell or run: source ~/.bashrc  # for bash");

    Ok(())
}
