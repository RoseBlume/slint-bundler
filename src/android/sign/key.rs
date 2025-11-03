
use std::io::{self, Write};

pub fn get_distinguished_names() -> String {
    let questions = [
        "What is your first and last name?", 
        "What is the name of your organizational unit?", 
        "What is the name of your organization?",
        "What is the name of your City or Locality?",
        "What is the name of your State or Province?",
        "What is the two-letter country code for this unit?"
    ];
    let mut answers = vec![];

    println!("Enter the distinguished name. Provide a single dot (.) to leave a sub-component empty or press ENTER to use the default value in braces.");
    
    for question in questions {
        let mut answer = String::new();
        print!("{}", question);
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut answer).expect("Failed to read line");
        answers.push(answer.trim().to_string());
    }
    format!(
        "CN={cn}, OU={ou}, O={o}, L={l}, ST={st} C={c}", 
        cn = questions[0],
        ou = questions[1],
        o = questions[2],
        l = questions[3],
        st = questions[4],
        c = questions[5]
    )
}