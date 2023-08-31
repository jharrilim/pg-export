pub struct Select {
    pub columns: Vec<String>,
    pub from: String,
    pub joins: Vec<Join>,
}

pub struct Join {
    pub table: String,
    pub on: On,
}

pub struct On {
    pub left: String,
    pub right: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complex_select() {}
}
