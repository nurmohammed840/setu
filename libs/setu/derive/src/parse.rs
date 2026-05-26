use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token::Paren,
    *,
};

pub struct AppName {
    pub as_token: Token![as],
    pub name: Ident,
    pub semi_token: Option<Token![;]>,
}

impl Parse for AppName {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            as_token: input.parse()?,
            name: input.parse()?,
            semi_token: input.parse()?,
        })
    }
}

pub struct RpcList {
    pub name: Option<AppName>,
    pub rpcs: Punctuated<Rpc, Token![;]>,
}

impl Parse for RpcList {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            name: input.peek(Token![as]).then(|| input.parse()).transpose()?,
            rpcs: Punctuated::parse_terminated(input)?,
        })
    }
}

pub struct Rpc {
    pub attrs: Vec<Attribute>,
    pub rpc_keyword: Token![fn],
    pub name: Ident,
    pub paren_token: Paren,
    pub args: Punctuated<Ident, Token![,]>,
    pub eq_token: Token![=],
    pub index: Lit,
}

impl Parse for Rpc {
    fn parse(input: ParseStream) -> Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let rpc_keyword = input.parse()?;
        let name = input.parse()?;

        let content;
        let paren_token = parenthesized!(content in input);
        let args = Punctuated::parse_terminated(&content)?;

        let eq_token = input.parse()?;
        let index = input.parse()?;

        Ok(Self {
            attrs,
            rpc_keyword,
            name,
            paren_token,
            args,
            eq_token,
            index,
        })
    }
}
