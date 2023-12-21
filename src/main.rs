#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]

use std::io::{self, StdoutLock, Write};

use crossterm::{event::{Event, KeyCode, self}, terminal::{disable_raw_mode, enable_raw_mode, ClearType, self}, cursor};
use rand::Rng;

#[derive(PartialEq)]
enum QuestionResult {
    Quit,
    Skip,
    Correct,
}

const fn num_to_char(num: u8) -> char {
    if num > 9 {
        '+'
    } else {
        (num + 0x30) as char
    }
}

fn position_cursor(lock: &mut StdoutLock, right: u16) {
    crossterm::execute!(
        lock,
        cursor::MoveUp(1),
        cursor::MoveToColumn(right),
    )
    .unwrap();
}

fn clear(lock: &mut StdoutLock) {
    crossterm::execute!(
        lock,
        cursor::MoveDown(1),
        terminal::Clear(ClearType::CurrentLine),
        cursor::MoveToPreviousLine(1),
        terminal::Clear(ClearType::CurrentLine),
        cursor::MoveToPreviousLine(1),
        terminal::Clear(ClearType::CurrentLine),
    )
    .unwrap();
}

fn display_guess(question: &str, correct: &str, guess: &str, lock: &mut StdoutLock) {
    clear(lock);

    let prompt = format!("{question} -> ");
    let whitespace = question.len() + 4;

    let mut lines: [String; 3] = [" ".repeat(whitespace), prompt, " ".repeat(whitespace)];

    let mut correct_iter = correct.as_bytes().iter();

    for char in guess.as_bytes() {
        if let Some(correct_char) = correct_iter.next() {
            if char == correct_char {
                lines[1].push_str("\x1b[0;32m");
                lines[0].push(' ');
                lines[2].push(' ');
            } else {
                lines[1].push_str("\x1b[0;31m");

                if char > correct_char {
                    lines[2].push(num_to_char(char - correct_char));
                    lines[0].push(' ');
                } else {
                    lines[0].push(num_to_char(correct_char - char));
                    lines[2].push(' ');
                }
            }
        }

        lines[1].push(*char as char);
    }

    lines[1].push_str("\x1b[0m");

    write!(lock, "{}", lines.join("\n\x1b[1G")).unwrap();
    position_cursor(lock, (guess.len() + whitespace) as u16);
    lock.flush().unwrap();
}

fn char_press(char: char, guess: &mut String, correct: &str, question: &str, mistakes: &mut [u16]) {
    if !char.is_ascii_alphabetic() {
        return;
    }

    guess.push(char);

    let last = guess.len() - 1;
    if last >= correct.len() {
        return;
    }

    let correct = correct.as_bytes();
    let question = question.as_bytes();

    if char != correct[last] as char {
        mistakes[question[last] as usize - 0x61] += 1;
    }
}

fn read(
    question: &str,
    correct: &str,
    lock: &mut StdoutLock,
    mistakes: &mut [u16],
) -> QuestionResult {
    let mut guess = String::new();

    loop {
        display_guess(question, correct, &guess, lock);

        if guess == correct {
            return QuestionResult::Correct;
        }

        let event = event::read().unwrap();

        if let Event::Key(key_event) = event {
            match key_event.code {
                KeyCode::Esc => return QuestionResult::Quit,
                KeyCode::Enter => return QuestionResult::Skip,
                KeyCode::Tab => return QuestionResult::Correct,
                KeyCode::Char(char) => char_press(char, &mut guess, correct, question, mistakes),
                KeyCode::Backspace => drop(guess.pop()),
                _ => {}
            }
        }
    }
}

fn main() {
    let list: Box<[&str]> = include_str!("../words.txt").lines().collect();

    enable_raw_mode().unwrap();

    println!("\n");

    let mut lock = io::stdout().lock();
    let mut rand = rand::thread_rng();

    let mut mistakes = [0; 26];

    loop {
        let range = if rand.gen_bool(0.5) {
            0..100
        } else {
            100..list.len()
        };

        let question = list[rand.gen_range(range)].to_ascii_lowercase();
        let ciphered: String = question
            .bytes()
            .map(|c| (if c >= 26 + 0x60 { 0x61 } else { c + 1 }) as char)
            .collect();

        let result = read(&question, &ciphered, &mut lock, &mut mistakes);

        match result {
            QuestionResult::Quit => break,
            QuestionResult::Skip => {}
            QuestionResult::Correct => {
                clear(&mut lock);
                println!("\x1b[0;32m{question}\x1b[0m -> {ciphered}\n\n");
            }
        }
    }

    disable_raw_mode().unwrap();
    clear(&mut lock);

    println!("\nmistakes:");
    for (i, count) in mistakes.iter().enumerate() {
        if *count == 0 {
            continue;
        }

        println!("{} - {}", (i as u8 + 0x61) as char, count);
    }
}
