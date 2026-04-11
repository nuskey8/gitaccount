use clap::{Parser, Subcommand};
use colour::blue_bold;
use colour::cyan_bold;
use colour::dark_gray;
use colour::gray;
use colour::gray_ln_bold;
use colour::green_bold;
use colour::red_bold;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[derive(Parser, Debug)]
#[command(name = "gitaccount")]
#[command(styles = clap_cargo::style::CLAP_STYLING)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Create a new account profile
    Create,

    /// Edit an existing account profile
    Edit {
        /// Account profile name
        profile: String,

        /// New git user.name
        #[arg(long)]
        name: Option<String>,

        /// New git user.email
        #[arg(long)]
        email: Option<String>,
    },

    /// Delete an account profile
    Delete {
        /// Account profile name
        profile: String,
    },

    /// Switch git account
    Switch {
        /// Account profile name
        profile: String,

        /// Update config in the current repository (.git/config)
        #[arg(long)]
        local: bool,

        /// Update config in the global git config (~/.gitconfig)
        #[arg(long = "global", conflicts_with = "local")]
        r#global: bool,
    },

    /// List configured accounts
    #[command(alias = "ls")]
    List,

    /// Clear git account
    Logout {
        /// Update config in the current repository (.git/config)
        #[arg(long)]
        local: bool,

        /// Update config in the global git config (~/.gitconfig)
        #[arg(long = "global", conflicts_with = "local")]
        r#global: bool,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Account {
    name: String,
    git_name: String,
    email: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct AccountStore {
    #[serde(default)]
    accounts: HashMap<String, Account>,
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Create => create_account(),
        Commands::Edit {
            profile,
            name,
            email,
        } => edit_account(&profile, name, email),
        Commands::Delete { profile: name } => delete_account(&name),
        Commands::Switch {
            profile: name,
            local,
            r#global,
        } => switch_account(&name, resolve_local_flag(local, r#global)),
        Commands::List => list_accounts(),
        Commands::Logout { local, r#global } => clear_config(resolve_local_flag(local, r#global)),
    };

    if let Err(err) = result {
        print_error(&err);
        std::process::exit(1);
    }
}

fn print_process(process_name: &str, message: &str) {
    const WIDTH: usize = 12;
    green_bold!("{:>width$}", process_name, width = WIDTH);
    println!(" {message}");
}

fn print_note(message: &str) {
    cyan_bold!("note");
    println!(": {message}");
}

fn print_error(message: &str) {
    red_bold!("error");
    println!(": {message}");
}

fn create_account() -> Result<(), String> {
    let profile_name: String =
        dialoguer::Input::with_theme(&dialoguer::theme::ColorfulTheme::default())
            .with_prompt("Profile name")
            .interact_text()
            .map_err(|e| format!("failed to read input: {e}"))?;
    let git_name: String =
        dialoguer::Input::with_theme(&dialoguer::theme::ColorfulTheme::default())
            .with_prompt("git user.name")
            .interact_text()
            .map_err(|e| format!("failed to read input: {e}"))?;
    let email: String = dialoguer::Input::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt("git user.email")
        .interact_text()
        .map_err(|e| format!("failed to read input: {e}"))?;

    if profile_name.is_empty() || git_name.is_empty() || email.is_empty() {
        return Err("all fields are required".to_string());
    }

    let mut store = load_store()?;
    if let Some(_) = store.accounts.get(&profile_name) {
        return Err(format!("account `{profile_name}` already exists"));
    }

    store.accounts.insert(
        profile_name.clone(),
        Account {
            name: profile_name.clone(),
            git_name,
            email,
        },
    );
    save_store(&store)?;

    print_process("Success", &format!("created account `{profile_name}`"));

    println!();
    print_note(&format!(
        "run `gitaccount switch <PROFILE>` to switch the account"
    ));
    Ok(())
}

fn edit_account(
    profile_name: &str,
    new_git_name: Option<String>,
    new_email: Option<String>,
) -> Result<(), String> {
    let mut store = load_store()?;
    let account = store
        .accounts
        .get_mut(profile_name)
        .ok_or_else(|| format!("account `{profile_name}` not found"))?;

    if new_git_name.is_none() && new_email.is_none() {
        let new_git_name: String =
            dialoguer::Input::with_theme(&dialoguer::theme::ColorfulTheme::default())
                .with_prompt("git user.name")
                .interact_text()
                .map_err(|e| format!("failed to read input: {e}"))?;

        let new_email: String =
            dialoguer::Input::with_theme(&dialoguer::theme::ColorfulTheme::default())
                .with_prompt("git user.email")
                .interact_text()
                .map_err(|e| format!("failed to read input: {e}"))?;

        if profile_name.is_empty() || new_git_name.is_empty() || new_email.is_empty() {
            return Err("all fields are required".to_string());
        }

        account.git_name = new_git_name;
        account.email = new_email;
    } else {
        if let Some(new_git_name) = new_git_name {
            if new_git_name.is_empty() {
                return Err("git user.name cannot be empty".to_string());
            }
            account.git_name = new_git_name;
        }

        if let Some(new_email) = new_email {
            if new_email.is_empty() {
                return Err("git user.email cannot be empty".to_string());
            }
            account.email = new_email;
        }
    }

    save_store(&store)?;

    print_process("Edited", &format!("`{}` account", profile_name));
    Ok(())
}

fn delete_account(profile_name: &str) -> Result<(), String> {
    let mut store = load_store()?;
    if !store.accounts.contains_key(profile_name) {
        return Err(format!("account `{profile_name}` not found"));
    }

    store.accounts.remove(profile_name);
    save_store(&store)?;

    print_process("Deleted", &format!("`{}` account", profile_name));

    Ok(())
}

fn switch_account(profile_name: &str, local: bool) -> Result<(), String> {
    let store = load_store()?;
    let account = store
        .accounts
        .get(profile_name)
        .ok_or_else(|| format!("account `{profile_name}` not found"))?;

    set_git_config("user.name", &account.git_name, local)?;
    set_git_config("user.email", &account.email, local)?;
    print_process(
        "Switched",
        &format!("`{}` account ({})", account.name, config_scope_label(local)),
    );
    Ok(())
}

fn list_accounts() -> Result<(), String> {
    let store = load_store()?;

    if store.accounts.is_empty() {
        print_error("no accounts found.");
        println!();
        print_note("run `gitaccount create` to add your first account.");
        return Ok(());
    }

    let current_name = get_git_global("user.name");
    let current_email = get_git_global("user.email");

    gray_ln_bold!("Accounts:");
    for account in store.accounts.values() {
        let is_active = current_name.as_deref() == Some(account.git_name.as_str())
            && current_email.as_deref() == Some(account.email.as_str());

        let width = std::cmp::max(
            6,
            store.accounts.values().map(|x| x.name.len()).max().unwrap(),
        );
        if is_active {
            let name = format!("{}", account.name);
            blue_bold!("  {:<width$}", name);
        } else {
            gray!("  {:<width$}", account.name);
        }
        dark_gray!("  {} <{}>", account.git_name, account.email);
        println!();
    }

    Ok(())
}

fn clear_config(local: bool) -> Result<(), String> {
    set_git_config("user.name", "", local)?;
    set_git_config("user.email", "", local)?;
    print_process(
        "Finished",
        &format!(
            "clear git {} user.name and user.email",
            config_scope_label(local)
        ),
    );
    Ok(())
}

fn set_git_config(key: &str, value: &str, local: bool) -> Result<(), String> {
    let scope = if local { "--local" } else { "--global" };

    let output = Command::new("git")
        .args(["config", scope, key, value])
        .output()
        .map_err(|e| format!("failed to run git: {e}"))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        if stderr.is_empty() {
            Err(format!("git config failed for key `{key}`"))
        } else {
            Err(format!("git config failed for key `{key}`: {stderr}"))
        }
    }
}

fn config_scope_label(local: bool) -> &'static str {
    if local { "local" } else { "global" }
}

fn resolve_local_flag(local: bool, _global: bool) -> bool {
    local
}

fn get_git_global(key: &str) -> Option<String> {
    let output = Command::new("git")
        .args(["config", "--global", "--get", key])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if value.is_empty() { None } else { Some(value) }
}

fn load_store() -> Result<AccountStore, String> {
    let path = accounts_file_path()?;
    if !path.exists() {
        return Ok(AccountStore::default());
    }

    let raw =
        fs::read_to_string(&path).map_err(|e| format!("failed to read {}: {e}", path.display()))?;
    toml::from_str(&raw).map_err(|e| format!("failed to parse {}: {e}", path.display()))
}

fn save_store(store: &AccountStore) -> Result<(), String> {
    let path = accounts_file_path()?;
    let toml_text = toml::to_string_pretty(store)
        .map_err(|e| format!("failed to serialize account store: {e}"))?;

    fs::write(&path, toml_text).map_err(|e| format!("failed to write {}: {e}", path.display()))
}

fn accounts_file_path() -> Result<PathBuf, String> {
    let home = env::var_os("HOME").ok_or_else(|| "HOME is not set".to_string())?;
    Ok(PathBuf::from(home).join(".gitaccount"))
}
