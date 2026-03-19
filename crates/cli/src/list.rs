use std::{
  fs,
  io::{self, Write},
  path::Path,
};

use cardbox::{list::MetaData, utils::puts};
use compact_str::format_compact;
use rayon::iter::{ParallelBridge, ParallelIterator};
use tap::{Pipe, Tap};

pub(crate) fn run(args: Option<&[String]>) -> io::Result<()> {
  use display_list_help as help;

  let args = match args {
    Some(&[]) | None => return print_json(&[".".into()]),
    Some(x) => x,
  };

  match args.first().map(|x| x.as_str()) {
    Some("-h") if !Path::new("-h").exists() => return help(),
    Some("--help") if !Path::new("--help").exists() => return help(),
    _ => {}
  }

  print_json(args)
}

fn print_json(args: &[String]) -> io::Result<()> {
  let mut lock = io::stdout().lock();
  lock.write_all(b"[\n")?;

  for (i, path) in args
    .iter()
    .filter_map(|x| match Path::new(x) {
      p if p.exists() => Some(p),
      _ => {
        eprintln!("// [WARN] path does not exist: {x}");
        None
      }
    })
    .enumerate()
  {
    if i != 0 {
      lock.write_all(b",\n")?
    }
    if path.is_file() {
      MetaData::new(path)?
        .with_id(format_compact!("==={i}==="))
        .to_json_pretty()?
        .pipe(|s| lock.write_all(s.as_bytes()))?;
      continue;
    }

    let entries = fs::read_dir(path)? //
      .filter_map(|x| x.ok())
      .enumerate()
      .par_bridge()
      .map(|(ii, entry)| {
        entry
          .path()
          .pipe(MetaData::new)
          .and_then(|data| {
            data
              .with_id(format_compact!("==={i}.{ii}==="))
              .to_json_pretty()
          })
          .map(|s| (ii, s))
      })
      .collect::<io::Result<Vec<_>>>()?
      .tap_mut(|v| v.sort_unstable_by_key(|(ii, _)| *ii));

    for (ii, s) in entries {
      if ii != 0 {
        lock.write_all(b",\n")?
      }
      lock.write_all(s.as_bytes())?
    }
  }

  lock.write_all(b"\n]\n")?;
  lock.flush()
}

fn display_list_help() -> io::Result<()> {
  r#"
Usage:
  list [/path/to/file] [/path/to/dir]

Examples:
  list .
  list /tmp
  list /tmp/a.txt
  list a.txt b.txt
  "#
  .pipe(puts)?;
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[ignore]
  #[test]
  fn list_files() -> io::Result<()> {
    let files = ["Cargo.toml", "tmp"].map(|s| s.to_owned());
    run(Some(&files))?;

    Ok(())
  }
}
