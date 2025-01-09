use std::collections::HashSet;

use anyhow::{anyhow, Result};
use indexmap::IndexMap;

type Parameters = IndexMap<(usize, usize), usize>;

#[derive(Debug)]
pub struct ParameterizedString<'a> {
    input: &'a str,
    parameters: Parameters,
    count: usize,
}

impl<'a> ParameterizedString<'a> {
    pub fn new(input: &'a str) -> Result<Self> {
        let (parameters, count) = Self::extract_parameters(input)?;

        Ok(Self {
            input,
            parameters,
            count,
        })
    }

    pub fn parameters(&self) -> usize {
        self.count
    }

    fn extract_parameters(input: &str) -> Result<(Parameters, usize)> {
        let mut parameters = IndexMap::new();
        let mut set = HashSet::new();
        let mut min = 0;
        let mut max = 0;

        let mut start = 0;
        while let Some(open_brace) = input[start..].find('{') {
            let start_index = start + open_brace;
            if let Some(close_brace) = input[start_index..].find('}') {
                let end_index = start_index + close_brace;
                let number_str = &input[start_index + 1..end_index];

                match number_str.parse::<usize>() {
                    Ok(number) => {
                        parameters.insert((start_index, end_index), number);
                        set.insert(number);
                        min = min.min(number);
                        max = max.max(number);
                    }
                    Err(_) => {}
                }

                start = end_index + 1;
            } else {
                return Err(anyhow!("Unmatched '{{' found."));
            }
        }

        if !set.is_empty() {
            for num in min..=max {
                if !set.contains(&num) {
                    return Err(anyhow!("Missing parameter: {{{num}}}"));
                }
            }
        }

        Ok((parameters, set.len()))
    }

    pub fn to_string<S>(&self, parameters: Vec<S>) -> Result<String>
    where
        S: AsRef<str>,
    {
        if parameters.len() != self.count {
            return Err(anyhow!(
                "Expected {} parameter{}, got {} parameter{}",
                self.parameters.len(),
                if self.parameters.len() == 1 { "" } else { "s" },
                parameters.len(),
                if parameters.len() == 1 { "" } else { "s" }
            ));
        }

        let mut result = String::new();

        let mut index = 0;
        for (&(start_idx, end_idx), &i) in &self.parameters {
            result += &self.input[index..start_idx];
            result += parameters[i].as_ref();

            index = end_idx + 1;
        }

        result += &self.input[index..];

        Ok(result)
    }
}
