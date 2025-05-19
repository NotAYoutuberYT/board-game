use std::{io::Write, path::PathBuf, str::FromStr};

use mini::Mini;
use parser::parse_instructions;
use rfd::FileDialog;
use village::{Village, VillageStatus};

mod mini;
mod parser;
mod village;

fn main() {
    let mut village = Village::new(6, 2, 2, 2);

    loop {
        let instructions;
        let starting_location;

        // get instructions for the mini
        loop {
            let file = match FileDialog::new()
                .set_title("Select mini code")
                .add_filter("mm code", &["mm", "txt"])
                .set_directory("/")
                .set_can_create_directories(true)
                .pick_file()
            {
                Some(file) => file,

                // if the file dialog fails (like it does on NixOS unfortunately), just
                // prompt from the command-line
                None => {
                    print!("Select file containing mini code: ");
                    std::io::stdout().flush().expect("falied to flush stdout");
                    let mut buffer = String::new();
                    std::io::stdin()
                        .read_line(&mut buffer)
                        .expect("failed to read stdin");

                    // if given an invalid path (such as an empty string), just ask again
                    match PathBuf::from_str(&buffer.trim()) {
                        Ok(path) => path,
                        Err(_) => continue,
                    }
                }
            };

            // if the we successfully parse instructions, move on.
            // otherwise, prompt the user again
            match parse_instructions(file) {
                Ok(ins) => {
                    instructions = ins;
                    break;
                }
                Err(error) => println!("please try again: {}", error),
            }
        }

        // get starting location
        loop {
            print!("Select starting location: ");
            std::io::stdout().flush().expect("failed to flush stdout");
            let mut buffer = String::new();
            std::io::stdin()
                .read_line(&mut buffer)
                .expect("failed to read stdin");

            // if we were given a valid u8, continue. otherwise, ask again
            match buffer.trim().parse::<u8>() {
                Ok(location) => {
                    if village.villager_exists(location) {
                        starting_location = location;
                        break;
                    } else {
                        println!("there is no villager at that location")
                    }
                }
                Err(e) => println!("that's not a valid number: {}", e),
            }
        }

        // run the mini and output the log
        let mut mini = Mini::new(starting_location, instructions, &village);
        mini.run_until_completion(&mut village);
        println!("\nMini log:");
        mini.log().iter().for_each(|log| println!("{:?}", log));

        // run the village night and handle winning/losing
        village.run_night();
        if village.status() != VillageStatus::Running {
            break;
        }

        // print information and continue to next iteration
        print!("Day complete. Press enter to continue... ");
        std::io::stdout().flush().expect("failed to flush stdout");
        let mut buffer = String::new();
        std::io::stdin()
            .read_line(&mut buffer)
            .expect("failed to read stdin");
        println!();
    }

    // print game overview
    match village.status() {
        VillageStatus::MurdersWon => println!("\nYou lose! All the villagers have died."),
        VillageStatus::VillagersWon => println!("\nYou win! All the murderers have died."),
        VillageStatus::Running => unreachable!(),
    }

    println!("\nThe village layout was:");
    let mut layout = village.layout();
    layout.sort_by(|a, b| a.label().cmp(&b.label()));
    layout.iter().for_each(|villager| {
        println!(
            "{}: {}",
            villager.label(),
            match villager.kind() {
                village::VillagerType::Normal => "normal villager",
                village::VillagerType::Strong(_) => "strong villager",
                village::VillagerType::Afraid => "afraid villager",
                village::VillagerType::Murderer => "murderer",
            }
        )
    });
}
