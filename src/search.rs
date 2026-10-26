use std::ops::Range;

#[derive(Default)]
pub(crate) struct RunText {
    pub(crate) buf: String,
    /// Where each span of the run starts in `buf`, and the leaf it came from.
    pub(crate) spans: Vec<(usize, usize)>,
}

impl RunText {
    /// Reads a run, as the text of each span and the leaf it came from.
    pub(crate) fn read<'a>(&mut self, run: impl Iterator<Item = (usize, &'a str)>) -> &str {
        self.buf.clear();
        self.spans.clear();

        for (leaf, text) in run {
            self.spans.push((self.buf.len(), leaf));
            self.buf.push_str(text);
        }

        &self.buf
    }

    /// Cuts `matched` up along the spans it covers, as each is painted on its
    /// own and knows only its own text.
    pub(crate) fn per_leaf(
        &self,
        matched: Range<usize>,
    ) -> impl Iterator<Item = (usize, Range<usize>)> {
        let from = self
            .spans
            .partition_point(|&(start, _)| start <= matched.start)
            .saturating_sub(1);

        self.spans[from..]
            .iter()
            .enumerate()
            .take_while(move |&(_, &(start, _))| start < matched.end)
            .map(move |(i, &(start, leaf))| {
                let end = match self.spans.get(from + i + 1) {
                    Some(&(next, _)) => next,
                    None => self.buf.len(),
                };

                (
                    leaf,
                    matched.start.max(start) - start..matched.end.min(end) - start,
                )
            })
    }
}
