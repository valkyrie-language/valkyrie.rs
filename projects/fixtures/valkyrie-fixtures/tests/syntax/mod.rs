//! syntax suite modules.

pub mod builder {
mod main;
    pub mod test_files {
    
    }
    
}

mod builder_eof;
mod comment_and_micro;
mod enumerate_iterator_parse;
pub mod formatter {
mod main;
}

mod incomplete_parse;
pub mod lexer {
mod main;
}

mod literals;
mod modifier_nodes;
mod namespace_using_parse;
pub mod parser {
mod main;
}

pub mod printer {
mod main;
}

mod system_builder;
mod template_micro_body;
mod template_trailing_newline;
mod turbofish_if_return;