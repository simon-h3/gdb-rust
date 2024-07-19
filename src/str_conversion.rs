/*
    Simon H - 2024
*/

// Function to convert the fixed-size char array to a String for display
pub fn char_print(fixed: &[char]) -> String {
    fixed.iter().collect::<String>().trim_end_matches('\0').to_string() // Convert to String and remove trailing null characters
}

// Function to populate a fixed-size char array with a &str
pub fn populate_fixed_chars(target: &mut [char], input_str: &str) {
    let chars: Vec<char> = input_str.chars().collect();
    let len = chars.len().min(target.len()); // Ensure we don't exceed target length
    for i in 0..len {
        target[i] = chars[i];
    }
}

// Function to convert a &str to a fixed-size char array
pub fn str_to_fixed_chars(input_str: &str) -> [char; 16] {
    let mut chars: [char; 16] = [' '; 16];                      // initialise empty (spaces)
    populate_fixed_chars(&mut chars, input_str);                // populate
chars                                                           // return
}
