use query::Query;
use Searcher;
use query::Weight;
use Result;
use schema::Term;
use std::collections::HashMap;
use schema::Type;
use levenshtein_automata::LevenshteinAutomatonBuilder;
use super::FuzzyTermWeight;
use error::ErrorKind;
use schema::Field;

const MAX_DISTANCE: u8 = 3;

lazy_static! {
    static ref LEVENSHTEIN_DFA_BUILDER_CACHE: HashMap<(u8, bool), LevenshteinAutomatonBuilder> = {
        let mut dfa_builder_cache = HashMap::new();
        for distance in 1u8..(MAX_DISTANCE + 1u8) {
            for has_transposition in [false, true].iter().cloned() {
                let dfa_builder = LevenshteinAutomatonBuilder::new(distance, has_transposition);
                dfa_builder_cache.insert((distance, has_transposition), dfa_builder);
            }
        }
        dfa_builder_cache
    };
}

#[derive(Debug)]
pub struct FuzzyTermQuery {
    term: Term,
    distance: u8,
    has_transposition: bool
}

impl Query for FuzzyTermQuery {
    fn weight(&self, searcher: &Searcher, scoring_enabled: bool) -> Result<Box<Weight>> {
        let schema = searcher.schema();
        let field = self.term.field();
        let field_entry = schema.get_field_entry(field);
        let actual_type = field_entry.field_type().value_type();
        let schema = searcher.schema();
        let field_entry= schema.get_field_entry(field);
        if actual_type != Type::Str {
            let err_msg = format!("`FuzzyTermQuery` requires {} to be a text field. (It is {:?})", field_entry.name(), actual_type);
            bail!(ErrorKind::SchemaError(err_msg));
        }
        assert_eq!(field_entry.field_type().value_type(), Type::Str, "FuzzyQuery requires a String field.");
        let term_value = self.term.text();
        if let Some(lev_dfa_builder) = LEVENSHTEIN_DFA_BUILDER_CACHE.get(&(self.distance, self.has_transposition)) {
            let dfa = lev_dfa_builder.build_dfa(term_value);
            Ok(Box::new(FuzzyTermWeight::new(self.term.clone(), dfa)))
        } else {
            bail!(ErrorKind::InvalidArgument(format!("Distance not handled : {}. Required to be <= {}", self.distance, MAX_DISTANCE)))
        }
    }
}


impl FuzzyTermQuery {
    pub fn new(term: Term, distance: u8, has_transposition: bool) -> FuzzyTermQuery {
        assert!(distance > 0u8);
        FuzzyTermQuery {
            term,
            distance,
            has_transposition
        }
    }

    pub fn field(&self) -> Field {
        self.term.field()
    }
}
