use super::*;
use crate::input::error::InputError;
use crate::input::{Input, InputBlockIterator};
use crate::query::JsonString;
use crate::result::InputRecorder;
use crate::FallibleIterator;

pub(crate) struct Constructor;

impl MemmemImpl for Constructor {
    type Classifier<'i, 'b, 'r, I, R> = SequentialMemmemClassifier<'i, 'b, 'r, I, R, BLOCK_SIZE>
    where
        I: Input + 'i,
        <I as Input>::BlockIterator<'i, 'r, BLOCK_SIZE, R>: 'b,
        R: InputRecorder<<I as Input>::Block<'i, BLOCK_SIZE>> + 'r,
        'i: 'r;

    fn memmem<'i, 'b, 'r, I, R>(
        input: &'i I,
        iter: &'b mut <I as Input>::BlockIterator<'i, 'r, BLOCK_SIZE, R>,
    ) -> Self::Classifier<'i, 'b, 'r, I, R>
    where
        I: Input,
        R: InputRecorder<<I as Input>::Block<'i, BLOCK_SIZE>>,
        'i: 'r,
    {
        Self::Classifier { input, iter }
    }
}

pub(crate) struct SequentialMemmemClassifier<'i, 'b, 'r, I, R, const N: usize>
where
    I: Input,
    R: InputRecorder<I::Block<'i, N>> + 'r,
{
    input: &'i I,
    iter: &'b mut I::BlockIterator<'i, 'r, N, R>,
}

impl<'i, 'b, 'r, I, R, const N: usize> SequentialMemmemClassifier<'i, 'b, 'r, I, R, N>
where
    I: Input,
    R: InputRecorder<I::Block<'i, N>> + 'r,
{
    #[inline]
    fn find_label_sequential(
        &mut self,
        label: &JsonString,
        mut offset: usize,
    ) -> Result<Option<(usize, I::Block<'i, N>)>, InputError> {
        let label_size = label.bytes_with_quotes().len();
        let first_c = if label.bytes().is_empty() {
            b'"'
        } else {
            label.bytes()[0]
        };

        while let Some(block) = self.iter.next()? {
            let res = block.iter().copied().enumerate().find(|&(i, c)| {
                let j = offset + i;
                c == first_c && j > 0 && self.input.is_member_match(j - 1, j + label_size - 2, label)
            });

            if let Some((res, _)) = res {
                return Ok(Some((res + offset - 1, block)));
            }

            offset += block.len();
        }

        Ok(None)
    }
}

impl<'i, 'b, 'r, I, R, const N: usize> Memmem<'i, 'b, 'r, I, N> for SequentialMemmemClassifier<'i, 'b, 'r, I, R, N>
where
    I: Input,
    R: InputRecorder<I::Block<'i, N>> + 'r,
{
    // Output the relative offsets
    fn find_label(
        &mut self,
        first_block: Option<I::Block<'i, N>>,
        start_idx: usize,
        label: &JsonString,
    ) -> Result<Option<(usize, I::Block<'i, N>)>, InputError> {
        if let Some(b) = first_block {
            if let Some(res) = shared::find_label_in_first_block(self.input, b, start_idx, label)? {
                return Ok(Some(res));
            }
        }
        let next_block_offset = self.iter.get_offset();

        self.find_label_sequential(label, next_block_offset)
    }
}
