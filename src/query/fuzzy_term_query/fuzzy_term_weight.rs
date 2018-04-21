use Term;
use levenshtein_automata::DFA;
use query::Weight;
use SegmentReader;
use Error;
use query::Scorer;
use super::FuzzyTermScorer;
use common::BitSet;
use query::BitSetDocSet;
use schema::IndexRecordOption;

pub struct FuzzyTermWeight {
    term: Term,
    dfa: DFA
}

impl FuzzyTermWeight {
    pub fn new(term: Term, dfa: DFA) -> FuzzyTermWeight {
        FuzzyTermWeight {
            term,
            dfa
        }
    }
}

impl Weight for FuzzyTermWeight {
    fn scorer(&self, reader: &SegmentReader) -> Result<Box<Scorer>, Error> {


        let max_doc = reader.max_doc();
        let mut doc_bitset = BitSet::with_max_value(max_doc);

        let inverted_index = reader.inverted_index(self.field);
        let term_dict = inverted_index.terms();
        let mut term_range = self.term_range(term_dict);
        while term_range.advance() {
            let term_info = term_range.value();
            let mut block_segment_postings = inverted_index
                .read_block_postings_from_terminfo(term_info, IndexRecordOption::Basic);
            while block_segment_postings.advance() {
                for &doc in block_segment_postings.docs() {
                    doc_bitset.insert(doc);
                }
            }
        }
        let doc_bitset = BitSetDocSet::from(doc_bitset);
        BitSetDocSet::from(doc_bitset)
    }
}