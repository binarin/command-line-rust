#[derive(Debug, Clone, PartialEq)]
pub enum CLIInput {
    StdIn,
    File(String),
}

#[cfg(test)]
mod tests {
    use super::*;
}
