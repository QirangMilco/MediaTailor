mod config;
mod font;
mod parser;
mod render;

use std::env;
use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use config::load_project_config;
use font::validate_project_fonts;
use parser::parse_mtc_file;
use render::render_canvas;

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let mut args = env::args().skip(1);
    let Some(command) = args.next() else {
        print_usage();
        bail!("missing command")
    };

    match command.as_str() {
        "render" => {
            let input = args
                .next()
                .map(PathBuf::from)
                .ok_or_else(|| anyhow::anyhow!("missing input .mtc file"))?;
            let output = args
                .next()
                .map(PathBuf::from)
                .ok_or_else(|| anyhow::anyhow!("missing output image path"))?;
            if args.next().is_some() {
                bail!("too many arguments for render command");
            }
            render_file(input, output)
        }
        "check" => {
            let input = args
                .next()
                .map(PathBuf::from)
                .ok_or_else(|| anyhow::anyhow!("missing input .mtc file"))?;
            if args.next().is_some() {
                bail!("too many arguments for check command");
            }
            let doc = parse_mtc_file(&input)?;
            let config = load_project_config(&input)?;
            validate_project_fonts(&config)?;
            println!(
                "ok: canvas `{}` => {}x{}, nodes={}",
                doc.name,
                doc.width,
                doc.height,
                doc.nodes.len()
            );
            Ok(())
        }
        "help" | "--help" | "-h" => {
            print_usage();
            Ok(())
        }
        other => {
            print_usage();
            bail!("unsupported command: {other}")
        }
    }
}


fn render_file(input: PathBuf, output: PathBuf) -> Result<()> {
    let doc = parse_mtc_file(&input)?;
    let config = load_project_config(&input)?;
    validate_project_fonts(&config)?;
    let image = render_canvas(&doc, &config)?;
    image
        .save(&output)
        .with_context(|| format!("failed to save output image: {}", output.display()))?;
    println!(
        "rendered canvas `{}` => {} ({}x{})",
        doc.name,
        output.display(),
        doc.width,
        doc.height
    );
    Ok(())
}

fn print_usage() {
    eprintln!("MediaTailor MTC prototype");
    eprintln!("Usage:");
    eprintln!("  cargo run -p mt-cli -- check <input.mtc>");
    eprintln!("  cargo run -p mt-cli -- render <input.mtc> <output.png>");
}
