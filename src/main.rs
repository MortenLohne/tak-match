use std::io::{BufWriter, Result};

use crate::cli::CliOptions;
use crate::engine::EngineBuilder;
use std::fs;
use std::sync::Mutex;
use taik::board::{Board, Move};

mod cli;
mod engine;
mod game;
mod r#match;
mod openings;
pub mod pgn_writer;
#[cfg(test)]
mod tests;
pub mod uci;

fn main() -> Result<()> {
    let cli_args = cli::parse_cli_arguments();

    let openings = match &cli_args.book_path {
        Some(path) => openings::openings_from_file(path)?,
        None => vec![vec![]],
    };

    run_match(openings, cli_args)?;

    Ok(())
}

fn run_match(openings: Vec<Vec<Move>>, cli_args: CliOptions) -> Result<()> {
    let engine_builders: Vec<EngineBuilder> = cli_args
        .engine_paths
        .iter()
        .zip(cli_args.engine_args.iter())
        .map(|(path, args)| EngineBuilder { path, args })
        .collect();

    let settings: r#match::TournamentSettings<Board> = r#match::TournamentSettings {
        concurrency: cli_args.concurrency,
        time: cli_args.time,
        increment: cli_args.increment,
        openings,
        num_minimatches: (cli_args.games + 1) / 2,
        pgn_writer: cli_args.pgnout.as_ref().map(|pgnout| {
            Mutex::new(r#match::PgnWriter::new(BufWriter::new(
                fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(pgnout)
                    .unwrap(),
            )))
        }),
    };

    println!("CLI args: {:?}", cli_args);
    println!("Settings: {:?}", settings);

    let _ = r#match::play_match(
        &settings,
        engine_builders[0].clone(),
        engine_builders[1].clone(),
    );
    Ok(())
}
