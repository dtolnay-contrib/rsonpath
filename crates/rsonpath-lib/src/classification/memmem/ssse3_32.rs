use super::{shared::mask_32, shared::vector_128, *};
use crate::{
    classification::mask::m32,
    input::{error::InputError, Input, InputBlock, InputBlockIterator},
    query::JsonString,
    result::InputRecorder,
    FallibleIterator,
};

const SIZE: usize = 32;

pub(crate) struct Ssse3MemmemClassifier32<'i, 'b, 'r, I, R>
where
    I: Input,
    R: InputRecorder<I::Block<'i, SIZE>> + 'r,
{
    input: &'i I,
    iter: &'b mut I::BlockIterator<'i, 'r, SIZE, R>,
}

impl<'i, 'b, 'r, I, R> Ssse3MemmemClassifier32<'i, 'b, 'r, I, R>
where
    I: Input,
    R: InputRecorder<I::Block<'i, SIZE>>,
    'i: 'r,
{
    #[inline]
    #[allow(dead_code)]
    pub(crate) fn new(input: &'i I, iter: &'b mut I::BlockIterator<'i, 'r, SIZE, R>) -> Self {
        Self { input, iter }
    }

    // Here we want to detect the pattern `"c"`
    // For interblock communication we need to bit of information that requires extra work to get obtained.
    // one for the block cut being `"` and `c"` and one for `"c` and `"`. We only deal with one of them.
    #[target_feature(enable = "ssse3")]
    unsafe fn find_letter(
        &mut self,
        label: &JsonString,
        mut offset: usize,
    ) -> Result<Option<(usize, I::Block<'i, SIZE>)>, InputError> {
        let classifier = vector_128::BlockClassifier128::new(label.bytes()[0], b'"');
        let mut previous_block: u32 = 0;

        while let Some(block) = self.iter.next()? {
            let (block1, block2) = block.halves();
            let classified1 = classifier.classify_block(block1);
            let classified2 = classifier.classify_block(block2);

            let mut first_bitmask = m32::combine_16(classified1.first, classified2.first);
            let second_bitmask = m32::combine_16(classified1.second, classified2.second);

            first_bitmask &= second_bitmask << 1 | 1; // we AND `"` bitmask with `c` bitmask to filter c's position in the stream following a `"`
                                                      // We should need the last bit of previous block. Instead of memoizing, we simply assume it is one.
                                                      // It could gives only add more potential match.

            if let Some(res) =
                mask_32::find_in_mask(self.input, label, previous_block, first_bitmask, second_bitmask, offset)
            {
                return Ok(Some((res, block)));
            }

            offset += SIZE;
            previous_block = first_bitmask >> (SIZE - 1);
        }

        Ok(None)
    }

    #[target_feature(enable = "ssse3")]
    #[inline]
    unsafe fn find_label_ssse3(
        &mut self,
        label: &JsonString,
        mut offset: usize,
    ) -> Result<Option<(usize, I::Block<'i, SIZE>)>, InputError> {
        if label.bytes().len() == 1 {
            return self.find_letter(label, offset);
        }

        let classifier = vector_128::BlockClassifier128::new(label.bytes()[0], label.bytes()[1]);
        let mut previous_block: u32 = 0;

        while let Some(block) = self.iter.next()? {
            let (block1, block2) = block.halves();
            let classified1 = classifier.classify_block(block1);
            let classified2 = classifier.classify_block(block2);

            let first_bitmask = m32::combine_16(classified1.first, classified2.first);
            let second_bitmask = m32::combine_16(classified1.second, classified2.second);

            if let Some(res) =
                mask_32::find_in_mask(self.input, label, previous_block, first_bitmask, second_bitmask, offset)
            {
                return Ok(Some((res, block)));
            }

            offset += SIZE;
            previous_block = first_bitmask >> (SIZE - 1);
        }

        Ok(None)
    }
}

impl<'i, 'b, 'r, I, R> Memmem<'i, 'b, 'r, I, SIZE> for Ssse3MemmemClassifier32<'i, 'b, 'r, I, R>
where
    I: Input,
    R: InputRecorder<I::Block<'i, SIZE>>,
    'i: 'r,
{
    // Output the relative offsets
    fn find_label(
        &mut self,
        first_block: Option<I::Block<'i, SIZE>>,
        start_idx: usize,
        label: &JsonString,
    ) -> Result<Option<(usize, I::Block<'i, SIZE>)>, InputError> {
        if let Some(b) = first_block {
            if let Some(res) = shared::find_label_in_first_block(self.input, b, start_idx, label)? {
                return Ok(Some(res));
            }
        }
        let next_block_offset = self.iter.get_offset();
        // SAFETY: target feature invariant
        unsafe { self.find_label_ssse3(label, next_block_offset) }
    }
}
