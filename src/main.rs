use mini::Mini;
use parser::parse_instructions;
use rfd::FileDialog;
use village::Village;

mod mini;
mod parser;
mod village;

fn main() {
    let mut village = Village::new(4, 2, 1);

    loop {
        let instructions;
        let starting_location;

        // get instructions
        loop {
            let file = FileDialog::new()
                .set_title("Select mini code")
                .add_filter("mm code", &["mm", "txt"])
                .set_directory("/")
                .set_can_create_directories(true)
                .pick_file()
                .expect("failed to get path");

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
            println!("Select starting location: ");
            let mut buffer = String::new();
            std::io::stdin()
                .read_line(&mut buffer)
                .expect("failed to read stdin");

            match buffer.trim().parse::<u8>() {
                Ok(location) => {
                    if village.villager_exists(location) {
                        starting_location = location;
                        break;
                    } else {
                        println!("there is no villager at that location")
                    }
                }
                Err(error) => println!("please try again: {}", error),
            }
        }

        let mut mini = Mini::new(starting_location, instructions, &village);
        mini.run_instruction(&mut village);
        mini.log().iter().for_each(|log| println!("{:?}", log));
    }
}
