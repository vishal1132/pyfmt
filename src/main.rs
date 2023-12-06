use std::ops::Add;
use xshell::{cmd, Shell};

const CMDS: &[(&str, fn(&Shell, sled::Db) -> anyhow::Result<()>)] =
    &[("run", run), ("clear", clear)];

fn run(sh: &Shell, db: sled::Db) -> anyhow::Result<()> {
    let branch = cmd!(sh, "git branch --show-current").read()?;
    let files = cmd!(sh, "git diff --cached --name-only --diff-filter=A").read()?;
    let mut new_files = files
        .split_whitespace()
        .filter(|&file| file.ends_with(".py"))
        .collect::<Vec<&str>>()
        .join(",");

    #[allow(clippy::collapsible_match)]
    if let Ok(ivec) = db.get(branch.clone()) {
        if let Some(val) = ivec {
            if !new_files.is_empty() && !new_files.ends_with(',') {
                new_files.push(',');
            }
            let result = String::from_utf8_lossy(val.as_ref()).to_string();
            new_files = new_files.add(result.as_str());
            new_files = new_files.add(",");
        }
    }

    if new_files.ends_with(',') {
        new_files.pop();
    }
    if new_files.is_empty() {
        return Ok(());
    }
    println!("linting files: {files}", files = new_files);

    db.insert(branch, new_files.as_bytes())?;

    new_files
        .split(',')
        .collect::<Vec<&str>>()
        .iter()
        .for_each(|file| {
            if let Err(err) = cmd!(sh, "black {file}").run() {
                eprintln!("error formatting {file} got error {err}");
            };
            if let Err(err) = cmd!(sh, "git add {file}").run() {
                eprintln!("error adding {file} to staged area got error {err}");
            };
        });

    Ok(())
}

fn clear(_: &Shell, db: sled::Db) -> anyhow::Result<()> {
    db.clear()?;
    println!("cleared db");
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let db_path = std::env::var("PYLINT_DB_PATH")
        .unwrap_or_else(|_| "/Users/vishal/.sled/pylint".to_string());
    let tree = sled::open(db_path)?;
    let sh = Shell::new()?;
    let mut args = std::env::args().skip(1);
    let arg = args.next().unwrap_or_default();

    let (_, run) = CMDS
        .iter()
        .find(|&&(name, _run)| name == arg)
        .ok_or_else(|| anyhow::format_err!("unknown cmd: `{arg}`"))?;

    run(&sh, tree)?;
    Ok(())
}
