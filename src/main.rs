use serde::{Deserialize, Serialize};
use std::io;
use std::io::Write;

// Config
const INITIAL_SCREEN_ID: &str = "0290922a-59ce-458b-8dbc-1c33f646580a";
const API_BASE: &str = "https://text-adventure.winsauce.com/api";

// Types
#[derive(Deserialize, Serialize)]
struct ClientGameState {
    inventory: Vec<String>,
}

impl ClientGameState {
    fn to_state_string(&self) -> String {
        let json = serde_json::to_string(&self).unwrap();
        let compressed_json = lz_str::compress_uri(&json)
            .iter()
            .map(|b| match char::from_u32(*b) {
                Some(c) => c,
                None => panic!("Non-char byte in compressed state string {}", b),
            })
            .collect::<String>();

        return compressed_json;
    }

    fn from_state_string(state_string: &str) -> ClientGameState {
        let raw_bytes = state_string.chars().map(|c| c as u32).collect::<Vec<u32>>();
        let json = lz_str::decompress_uri(&raw_bytes).expect(&format!(
            "Failed to decompress raw state string: {}",
            &state_string
        ));
        return serde_json::from_str::<ClientGameState>(&json)
            .expect(&format!("Failed to deserialise JSON state: {}", &json));
    }
}

#[derive(Deserialize)]
struct GameScreenDto {
    id: String,
    body: Vec<String>,
}

#[derive(Deserialize, Serialize)]
struct SubmitCommandDto {
    /** The screen the player is currently on */
    #[serde(rename = "contextScreenId")]
    context_screen_id: String,
    /** The command being submitted */
    #[serde(rename = "command")]
    command: String,
    /** Current state in the frontend */
    #[serde(rename = "state")]
    state: String,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum SubmitCommandResponse {
    SubmitCommandPrintMessageSuccess(SubmitCommandPrintMessageSuccessDto),
    SubmitCommandNavigationSuccess(SubmitCommandNavigationSuccessDto),
    SubmitCommandFailure(SubmitCommandFailureDto),
}

#[derive(Deserialize)]
struct SubmitCommandPrintMessageSuccessDto {
    #[serde(rename = "success")]
    success: bool,
    #[serde(rename = "type")]
    command_action_type: String,
    #[serde(rename = "printMessage")]
    print_message: Vec<String>,
    #[serde(rename = "state")]
    state: String,
    #[serde(rename = "itemsAdded")]
    items_added: Vec<String>,
    #[serde(rename = "itemsRemoved")]
    items_removed: Vec<String>,
}

#[derive(Deserialize)]
struct SubmitCommandNavigationSuccessDto {
    #[serde(rename = "success")]
    success: bool,
    #[serde(rename = "type")]
    command_action_type: String,
    #[serde(rename = "screen")]
    screen: GameScreenDto,
    #[serde(rename = "state")]
    state: String,
    #[serde(rename = "itemsAdded")]
    items_added: Vec<String>,
    #[serde(rename = "itemsRemoved")]
    items_removed: Vec<String>,
}

#[derive(Deserialize)]
struct SubmitCommandFailureDto {
    #[serde(rename = "success")]
    success: bool,
    #[serde(rename = "message")]
    message: String,
}

struct Game {
    current_screen: GameScreenDto,
    current_state: ClientGameState,
    client: reqwest::blocking::Client,
}

impl Game {
    fn new() -> Game {
        return Game {
            current_screen: GameScreenDto {
                id: String::new(),
                body: Vec::new(),
            },
            current_state: ClientGameState {
                inventory: Vec::new(),
            },
            client: reqwest::blocking::Client::new(),
        };
    }

    fn get_screen_by_id(&self, screen_id: &str) -> Option<GameScreenDto> {
        let request_url = format!("{API_BASE}/screen/{screen_id}");
        let response = self.client.get(&request_url).send().unwrap();
        return response.json::<GameScreenDto>().ok();
    }

    fn submit_command(&self, command: &str) -> SubmitCommandResponse {
        let request: SubmitCommandDto = SubmitCommandDto {
            context_screen_id: self.current_screen.id.clone(), // @TODO is `clone()` the right answer here? don't want to move `id`
            command: String::from(command),
            state: self.current_state.to_state_string(),
        };

        let response = self
            .client
            .post(format!("{API_BASE}/command"))
            .json(&request)
            .send()
            .unwrap();

        return response.json::<SubmitCommandResponse>().unwrap();
    }
}

fn main() {
    let mut game = Game::new();
    game.current_screen = game.get_screen_by_id(INITIAL_SCREEN_ID).unwrap();

    // Print initial screen
    for line in &game.current_screen.body {
        println!("{}", line);
    }

    let mut user_input: String = String::new();
    loop {
        user_input.clear();

        // Prompt character
        print!("\n> ");
        io::stdout().flush().unwrap();

        // Read from stdin
        io::stdin().read_line(&mut user_input).unwrap();

        // Evaluate input
        match user_input.trim() {
            "/inventory" => {
                // Print inventory
                println!("Current inventory:");
                for inventory_item in &game.current_state.inventory {
                    println!("  {}", &inventory_item);
                }
            }
            "/screen-id" | "/screen" => {
                // Print the current screen's ID
                println!("{}", &game.current_screen.id);
            }
            "/look" | "/whereami" | "/where" | "/repeat" | "/again" => {
                // Re-print the current screen
                for line in &game.current_screen.body {
                    println!("{}", &line);
                }
            }
            "/help" | "/?" => {
                // Print help text
                print_help_text();
            }
            "/exit" | "/quit" => {
                // Exit game
                break;
            }
            _ => {
                // Anything else is treated as a command
                match game.submit_command(&user_input.trim()) {
                    SubmitCommandResponse::SubmitCommandPrintMessageSuccess(dto) => {
                        // Message
                        for line in &dto.print_message {
                            println!("{}", line);
                        }
                        // Items added
                        if dto.items_added.len() > 0 {
                            println!("Items added:");
                            for item_name in &dto.items_added {
                                println!("+ {}", &item_name);
                            }
                        }
                        // Items removed
                        if dto.items_removed.len() > 0 {
                            println!("Items removed:");
                            for item_name in &dto.items_removed {
                                println!("- {}", &item_name);
                            }
                        }

                        // Update game's state
                        game.current_state = ClientGameState::from_state_string(&dto.state);
                    }
                    SubmitCommandResponse::SubmitCommandNavigationSuccess(dto) => {
                        // Print new game screen
                        for line in &dto.screen.body {
                            println!("{}", &line);
                        }

                        // Items added
                        if dto.items_added.len() > 0 {
                            println!("Items added:");
                            for item_name in &dto.items_added {
                                println!("+ {}", &item_name);
                            }
                        }
                        // Items removed
                        if dto.items_removed.len() > 0 {
                            println!("Items removed:");
                            for item_name in &dto.items_removed {
                                println!("- {}", &item_name);
                            }
                        }

                        // Update game's state
                        game.current_state = ClientGameState::from_state_string(&dto.state);
                        game.current_screen = dto.screen;
                    }
                    SubmitCommandResponse::SubmitCommandFailure(dto) => {
                        // Failure - did not match any command
                        println!("{}", &dto.message);
                    }
                }
            }
        }
    } // loop
}

fn print_help_text() {
    println!(
        "\
List of commands:
/inventory
    List your inventory

/screen-id
(alias: /screen)
    Print the current screen's
    id (useful when creating a
    new screen)

/look
(alias: /whereami)
(alias: /where)
(alias: /repeat)
(alias: /again)
    Print the current screen
    again

/help
(alias: /?)
    Print this help message

/exit
(alias: /quit)
    Quit the game"
    );
}
