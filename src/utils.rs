#[derive(PartialEq)]
pub enum QuoteState {
    None,
    Literal,
    RawLiteral
}

#[derive(PartialEq)]
pub enum CommentState {
    None,
    Single,
    Multi
}
#[derive(PartialEq)]
pub enum DelimiterState {
    Paren,
    Bracket,
    Brace
}

pub struct Quote {
    pub state: QuoteState,
    pub precede_escape: bool,
    pub precede_raw_quote: bool,
    pub changed: bool,
    pub num_delim: u32,
    pub cur_delim: u32
}

pub struct Comment {
    pub state: CommentState,
    pub precede_start: bool,
    pub precede_end: bool,
    pub changed: bool,
    pub depth: u32
}

pub struct Delimiters {
    pub state: Vec<DelimiterState>,
    pub is_end: bool
}

impl Delimiters {
    pub fn new() -> Self {
        Self { state: Vec::new(), is_end: false }
    }

    pub fn update(&mut self, ch: char) {
        if let Some(begin_delim) = match ch {
            '(' => Some(DelimiterState::Paren),
            '[' => Some(DelimiterState::Bracket),
            '{' => Some(DelimiterState::Brace),
            _ => None
        } {
            self.state.push(begin_delim);
        }
        else if let Some(end_delim) = match ch {
            ')' => Some(DelimiterState::Paren),
            ']' => Some(DelimiterState::Bracket),
            '}' => Some(DelimiterState::Brace),
            _ => None
        } {
            let last = self.state.last();
            if last.is_none() || *last.unwrap() != end_delim {
                match end_delim {
                    DelimiterState::Paren => {
                        panic!("Failed to find closing delimiter )");
                    },
                    DelimiterState::Bracket => {
                        panic!("Failed to find closing delimiter ]");
                    },
                    DelimiterState::Brace => {
                        panic!("Failed to find closing delimiter }");
                    }
                }
            }
            self.state.pop();
            if self.state.is_empty() {
                self.is_end = true;
            }
        }
    }
}

impl Quote {
    pub const fn new() -> Self {
        Self {
            state: QuoteState::None,
            precede_escape: false,
            precede_raw_quote: false,
            changed: false,
            num_delim: 0,
            cur_delim: 0
        }
    }

    pub fn update(&mut self, ch: char) {
        self.changed = false;
        match self.state {
            QuoteState::None => { // We are in raw text right now
                if self.precede_raw_quote {
                    match ch {
                        '#' => {
                            self.num_delim += 1;
                        },
                        '"' => {
                            self.precede_raw_quote = false;
                            self.state = QuoteState::RawLiteral;
                            self.changed = true;
                        },
                        _ => {
                            self.num_delim = 0;
                            self.precede_raw_quote = false;
                        }
                    }
                }
                else {
                    match ch {
                        'r' => {
                            self.precede_raw_quote = true;
                        },
                        '"' => {
                            self.state = QuoteState::Literal;
                            self.changed = true;
                        }
                        _ => {}
                    }
                }
            },
            QuoteState::Literal => { // There are no delimiters
                if self.precede_escape {
                    self.precede_escape = false; // ignore the char
                }
                else if ch == '"' {
                    self.state = QuoteState::None;
                    self.changed = true;
                }
                else if ch == '\\' {
                    self.precede_escape = true;
                }
            },
            QuoteState::RawLiteral => {
                if ch == '#' {
                    self.cur_delim += 1;
                }
                else {
                    self.cur_delim = 0;
                }
                if self.cur_delim == self.num_delim {
                    self.state = QuoteState::None;
                    self.changed = true;
                    self.cur_delim = 0;
                    self.num_delim = 0;
                }
            }
        }
    }
}

impl Comment {
    pub const fn new() -> Self {
        Self {
            state: CommentState::None,
            precede_start: false,
            precede_end: false,
            changed: false,
            depth: 0
        }
    }

    pub fn update(&mut self, ch: char) {
        self.changed = false;
        match self.state {
            CommentState::None => {
                if self.precede_start {
                    match ch {
                        '/' => {
                            self.state = CommentState::Single;
                            self.changed = true;
                        },
                        '*' => {
                            self.state = CommentState::Multi;
                            self.depth = 1;
                            self.changed = true;
                        },
                        _ => {}
                    }
                    self.precede_start = false;
                }
                else {
                    match ch {
                        '/' => {
                            self.precede_start = true;
                        },
                        _ => {}
                    }
                }
            },
            CommentState::Single => {
                match ch {
                    '\n' => {
                        self.state = CommentState::None;
                        self.changed = true;
                    },
                    '\r' => {
                        self.state = CommentState::None;
                        self.changed = true;
                    },
                    _ => {}
                }
            },
            CommentState::Multi => {
                let mut _ch = ch;
                if self.precede_start {
                    if _ch == '*' {
                        _ch = '\0';
                        self.depth += 1;
                    }
                    self.precede_start = false;
                }
                else if self.precede_end {
                    if _ch == '/' {
                        self.depth -= 1;
                        _ch = '\0';
                    }
                    self.precede_end = false;
                }

                if self.depth == 0 { self.state = CommentState::None; self.changed = true; }
                else if _ch == '/' { self.precede_start = true; }
                else if _ch == '*' { self.precede_end = true; }
            }
        }
    }
}

pub fn remove_comments(input: String) -> String {
    let mut chars = input.chars();
    let mut quote = Quote::new();
    let mut comment = Comment::new();
    let mut output = Vec::new();
    loop {
        let ch = chars.next();
        if ch.is_none() {
            break;
        }
        let ch = ch.unwrap();

        match comment.state {
            CommentState::None => {
                match quote.state {
                    QuoteState::None => {
                        comment.update(ch);
                    },
                    _ => { comment.update('\0'); }
                }
                quote.update(ch);
                output.push(ch);
            },
            _ => {
                if comment.changed {
                    output.pop();
                    output.pop();
                }
                comment.update(ch);
                quote.update('\0');
            }
        }
    }

    output.into_iter().collect()
}

pub fn remove_whitespace(input: String) -> String {
    let mut chars = input.chars();
    let mut quote = Quote::new();
    let mut comment = Comment::new();
    let mut output = Vec::new();

    loop {
        let ch = chars.next();
        if ch.is_none() {
            break;
        }
        let ch = ch.unwrap();
        
        quote.update(ch);
        comment.update(ch);

        let mut should_exclude = true;

        match quote.state {
            QuoteState::None => {
                if quote.changed {
                    should_exclude = false;
                }
            },
            _ => {
                should_exclude = false;
            }
        }

        match comment.state {
            CommentState::None => {
                if comment.changed {
                    should_exclude = false;
                }
            },
            _ => {
                should_exclude = false;
            }
        }

        if should_exclude {
            if !ch.is_whitespace() {
                output.push(ch);
            }
        }
        else {
            output.push(ch);
        }
    }
    output.into_iter().collect()
}

pub fn get_block(input: String) -> (usize, String) {
    let mut chars = input.chars();
    let mut output = Vec::new();
    let mut delim = Delimiters::new();
    let mut comment = Comment::new();
    let mut quote = Quote::new();
    let mut count = 0;
    let mut delim_found = false;

    loop {
        let ch = chars.next();
        if ch.is_none() {
            break;
        }
        count += 1;
        let ch = ch.unwrap();

        quote.update(ch);
        comment.update(ch);

        if quote.state == QuoteState::None && comment.state == CommentState::None {
            delim.update(ch);
        }
        else {
            delim.update('\0');
        }
        if !delim_found {
            if !delim.state.is_empty() {
                delim_found = true;
                output.push(ch);
            }
            else {
                if comment.state == CommentState::None {
                    output.push(ch);
                }
                else if comment.changed {
                    output.pop();
                }
            }
        }
        else {
            output.push(ch);
        }
        if delim.is_end {
            break;
        }
    }

    (count, output.into_iter().collect())
}

pub fn extract_internal_block(input: String) -> String {
    let mut chars = input.chars();
    let mut output = Vec::new();
    let mut delim = Delimiters::new();
    let mut comment = Comment::new();
    let mut quote = Quote::new();

    let mut delimiter_found = false;

    loop {
        let ch = chars.next();
        if ch.is_none() {
            break;
        }
        let ch = ch.unwrap();

        quote.update(ch);
        comment.update(ch);

        if quote.state == QuoteState::None && comment.state == CommentState::None {
            delim.update(ch);
        }
        else {
            delim.update('\0');
        }

        if delim.is_end {
            break;
        }
        if !delimiter_found {
            if !delim.state.is_empty() {
                delimiter_found = true;
            }
        }
        else {
            output.push(ch);
        }
    }
    output.into_iter().collect()
}

pub fn remove_preceding_whitespace(input: String) -> String {
    input.chars().skip_while(|c| c.is_whitespace()).collect()
}

pub fn remove_preceding_punctuation(input: String) -> String {
    input.chars().skip_while(|c| c.is_ascii_punctuation()).collect()
}

pub fn remove_preceding_unused(input: String) -> String {
    input.chars().skip_while(|c| c.is_ascii_punctuation() || c.is_whitespace()).collect()
}

pub fn space_separate_args(block: String) -> String {
    let mut chars = block.chars();
    let mut quote = Quote::new();
    let mut output = Vec::new();

    let ch = chars.next();
    if ch.is_none() {
        return String::new();
    }
    let ch = ch.unwrap();
    output.push(ch);
    quote.update(ch);
    loop {
        let ch = chars.next();
        if ch.is_none() {
            break;
        }
        let mut ch = ch.unwrap();
        if quote.state == QuoteState::None {
            if ch == ',' {
                output.push(ch);
                ch = ' ';
            }
        }
        output.push(ch);
        quote.update(ch);
    }
    output.into_iter().collect()
}

pub fn find_macro_positions(input: String) -> Vec<usize> {
    let mut ret = Vec::new();
    let mut chars = input.chars();

    let mut is_precede = false;
    let mut current_pos = 0;

    let mut quote = Quote::new();
    let mut comment = Comment::new();

    loop {
        let ch = chars.next();
        if ch.is_none() {
            break;
        }
        let ch = ch.unwrap();
        quote.update(ch);
        comment.update(ch);

        if quote.state == QuoteState::None && comment.state == CommentState::None {
            if is_precede {
                if ch == '[' {
                    ret.push(current_pos - 1);
                }
                is_precede = false;
            }
            else {
                is_precede = ch == '#';
            }
        }
        current_pos += 1;
    }

    ret
}

pub fn get_unqualified_name(qualified: String) -> String {
    let qualified = remove_preceding_whitespace(qualified);
    let mut result = qualified.clone();
    loop {
        if let Some(off) = result.find("::") {
            result = result.chars().skip(off + "::".len()).collect();
        }
        else if let Some(off) = result.find(".") {
            result = result.chars().skip(off + ".".len()).collect();
        }
        else {
            break;
        }
    }
    result
}