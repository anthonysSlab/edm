pub fn lines_enumerated(lines: &[String]) {
    let num_digits = lines.len().to_string().len();

    for (i, line) in lines.iter().enumerate() {
        let i = (i+1).to_string();
        print!("{}{}{}", i, " ".repeat(num_digits + 3 - i.len()), line);
    }
}
