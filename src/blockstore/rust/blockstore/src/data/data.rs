use anyhow::{ensure, Result};
use std::fmt::Debug;
use std::ops::{Bound, Deref, DerefMut, Range, RangeBounds};

/// An instance of data owns a block of data. It implements `AsRef<[u8]>` and `AsMut<[u8]>` to allow
/// borrowing that data, and it has a [Data::into_subregion] function that cuts away bytes at either
/// end of the block and returns a [Data] instance that (semantically) owns a subrange of the original
/// [Data] instance. This works without copying. Implementation wise, the new instance still owns and
/// holds all of the data, just the accessors got limited to a smaller subrange.
///  
/// That means this struct is great if you need to handle data blocks, cut away headers and pass ownership
/// of the remaining data on to something else without having to copy it. The downside is that the
/// header data isn't freed up - as long as any subregion of the original data exists somewhere,
/// the whole data has to be kept in memory.
#[derive(Clone)]
pub struct Data {
    storage: Vec<u8>,
    // region stores the subregion in the vector that we care for.
    // TODO We're probably safer with an invariant that 0 <= range.start <= range.end <= storage.len(). Otherwise we'd have to think about the other case everywhere.
    region: Range<usize>,
}

impl Data {
    /// Return the length of the [Data] instance (or if it is a subregion, length of the subregion)
    pub fn len(&self) -> usize {
        self.region.len()
    }

    /// Return a [Data] instance that semantically only represents a subregion of the original instance.
    /// Using any data accessors like `AsRef<[u8]>` or `AsMut<[u8]>` on the new instance will behave
    /// as if the instance only owned the subregion.
    ///
    /// Creating subregions is super fast and does not incur a copy.
    /// Note, however, that this is implemented by keeping all of the original data in memory and just
    /// changing the behavior of the accessors. The memory will only be freed once the subregion instance
    /// gets dropped.
    pub fn into_subregion(self, range: impl RangeBounds<usize> + Debug) -> Self {
        let start_bound_diff = match range.start_bound() {
            Bound::Unbounded => 0,
            Bound::Included(&x) => x,
            Bound::Excluded(&x) => x + 1,
        };
        let panic_end_out_of_bounds = || {
            panic!(
                "Range end out of bounds. Tried to access subregion {:?} for a Data instance of length {}",
                range,
                self.region.len(),
            );
        };
        let end_bound_diff = match range.end_bound() {
            Bound::Unbounded => 0,
            Bound::Included(&x) => self
                .region
                .len()
                .checked_sub(x + 1)
                .unwrap_or_else(panic_end_out_of_bounds),
            Bound::Excluded(&x) => self
                .region
                .len()
                .checked_sub(x)
                .unwrap_or_else(panic_end_out_of_bounds),
        };
        Self {
            storage: self.storage,
            region: Range {
                start: self.region.start + start_bound_diff,
                end: self.region.end - end_bound_diff,
            },
        }
    }

    /// Grow the subregion by adding more bytes to the beginning or end of the current region.
    /// This is the inverse of [Data::into_subregion].
    ///
    /// This function does not copy and will only succeed if there is enough space in the storage.
    // TODO Test
    pub fn grow_region(self, add_prefix_bytes: usize, add_suffix_bytes: usize) -> Result<Data> {
        ensure!(add_prefix_bytes <= self.region.start, "Tried to grow a Data object beyond its bounds. It only has {} prefix bytes available but {} were requested.", self.region.start, add_prefix_bytes);
        ensure!(add_suffix_bytes + self.region.end <= self.storage.len(), "Tried to grow a Data object beyond its bounds. It only has {} suffix bytes available but {} were requested.", self.storage.len().saturating_sub(self.region.end), add_suffix_bytes);
        Ok(Self {
            storage: self.storage,
            region: Range {
                start: self.region.start - add_prefix_bytes,
                end: self.region.end + add_suffix_bytes,
            },
        })
    }

    pub fn available_prefix_bytes(&self) -> usize {
        self.region.start
    }

    pub fn available_suffix_bytes(&self) -> usize {
        self.storage.len() - self.region.end
    }
}

impl From<Vec<u8>> for Data {
    /// Create a new [Data] object from a given `Vec<[u8]>` allocation.
    fn from(data: Vec<u8>) -> Data {
        let len = data.len();
        Self {
            storage: data,
            region: 0..len,
        }
    }
}

impl AsRef<[u8]> for Data {
    fn as_ref(&self) -> &[u8] {
        &self.storage[self.region.clone()]
    }
}

impl AsMut<[u8]> for Data {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.storage[self.region.clone()]
    }
}

// TODO Test
impl Deref for Data {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        self.as_ref()
    }
}

// TODO Test
impl DerefMut for Data {
    fn deref_mut(&mut self) -> &mut [u8] {
        self.as_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{rngs::StdRng, RngCore, SeedableRng};

    fn data_region(size: usize, seed: u64) -> Vec<u8> {
        let mut rng = StdRng::seed_from_u64(seed);
        let mut res = vec![0; size];
        rng.fill_bytes(&mut res);
        res
    }

    #[test]
    fn given_fullrangedata_when_callingasref() {
        let data: Data = data_region(1024, 0).into();
        assert_eq!(data.as_ref(), &data_region(1024, 0));
    }

    #[test]
    fn given_fullrangedata_when_callingasmut() {
        let mut data: Data = data_region(1024, 0).into();
        assert_eq!(data.as_mut(), &data_region(1024, 0));
    }

    #[test]
    fn given_fullrangedata_when_gettingavailablebytes() {
        let data: Data = data_region(1024, 0).into();
        assert_eq!(0, data.available_prefix_bytes());
        assert_eq!(0, data.available_suffix_bytes());
    }

    // TODO Implement this feature first
    // #[test]
    // fn given_fullrangedata_when_slicingfullrange() {
    //     let data: Data = data_region(1024, 0).into();
    //     assert_eq!(&data[..], &data_region(1024, 0));
    // }

    #[test]
    fn given_fullsubregiondata_when_callingasref() {
        let data: Data = data_region(1024, 0).into();
        let subdata = data.into_subregion(..);
        assert_eq!(subdata.as_ref(), &data_region(1024, 0));
    }

    #[test]
    fn given_fullsubregiondata_when_callingasmut() {
        let data: Data = data_region(1024, 0).into();
        let mut subdata = data.into_subregion(..);
        assert_eq!(subdata.as_mut(), &data_region(1024, 0));
    }

    #[test]
    fn given_fullsubregiondata_when_gettingavailablebytes() {
        let data: Data = data_region(1024, 0).into();
        let subdata = data.into_subregion(..);
        assert_eq!(0, subdata.available_prefix_bytes());
        assert_eq!(0, subdata.available_suffix_bytes());
    }

    #[test]
    fn given_openendsubregiondata_when_callingasref() {
        let data: Data = data_region(1024, 0).into();
        let subdata = data.into_subregion(5..);
        assert_eq!(subdata.as_ref(), &data_region(1024, 0)[5..]);
    }

    #[test]
    fn given_openendsubregiondata_when_callingasmut() {
        let data: Data = data_region(1024, 0).into();
        let mut subdata = data.into_subregion(5..);
        assert_eq!(subdata.as_mut(), &data_region(1024, 0)[5..]);
    }

    #[test]
    fn given_openendsubregiondata_when_gettingavailablebytes() {
        let data: Data = data_region(1024, 0).into();
        let subdata = data.into_subregion(5..);
        assert_eq!(5, subdata.available_prefix_bytes());
        assert_eq!(0, subdata.available_suffix_bytes());
    }

    #[test]
    fn given_openbeginningexclusivesubregiondata_when_callingasref() {
        let data: Data = data_region(1024, 0).into();
        let subdata = data.into_subregion(..1000);
        assert_eq!(subdata.as_ref(), &data_region(1024, 0)[..1000]);
    }

    #[test]
    fn given_openbeginningexclusivesubregiondata_when_callingasmut() {
        let data: Data = data_region(1024, 0).into();
        let mut subdata = data.into_subregion(..1000);
        assert_eq!(subdata.as_mut(), &data_region(1024, 0)[..1000]);
    }

    #[test]
    fn given_openbeginningexclusivesubregiondata_when_gettingavailablebytes() {
        let data: Data = data_region(1024, 0).into();
        let subdata = data.into_subregion(..1000);
        assert_eq!(0, subdata.available_prefix_bytes());
        assert_eq!(24, subdata.available_suffix_bytes());
    }

    #[test]
    fn given_openbeginninginclusivesubregiondata_when_callingasref() {
        let data: Data = data_region(1024, 0).into();
        let subdata = data.into_subregion(..=1000);
        assert_eq!(subdata.as_ref(), &data_region(1024, 0)[..=1000]);
    }

    #[test]
    fn given_openbeginninginclusivesubregiondata_when_callingasmut() {
        let data: Data = data_region(1024, 0).into();
        let mut subdata = data.into_subregion(..=1000);
        assert_eq!(subdata.as_mut(), &data_region(1024, 0)[..=1000]);
    }

    #[test]
    fn given_openbeginninginclusivesubregiondata_when_gettingavailablebytes() {
        let data: Data = data_region(1024, 0).into();
        let subdata = data.into_subregion(..=1000);
        assert_eq!(0, subdata.available_prefix_bytes());
        assert_eq!(23, subdata.available_suffix_bytes());
    }

    #[test]
    fn given_exclusivesubregiondata_when_callingasref() {
        let data: Data = data_region(1024, 0).into();
        let subdata = data.into_subregion(5..1000);
        assert_eq!(subdata.as_ref(), &data_region(1024, 0)[5..1000]);
    }

    #[test]
    fn given_exclusivesubregiondata_when_callingasmut() {
        let data: Data = data_region(1024, 0).into();
        let mut subdata = data.into_subregion(5..1000);
        assert_eq!(subdata.as_mut(), &data_region(1024, 0)[5..1000]);
    }

    #[test]
    fn given_exclusivesubregiondata_when_gettingavailablebytes() {
        let data: Data = data_region(1024, 0).into();
        let subdata = data.into_subregion(5..1000);
        assert_eq!(5, subdata.available_prefix_bytes());
        assert_eq!(24, subdata.available_suffix_bytes());
    }

    #[test]
    fn given_inclusivesubregiondata_when_callingasref() {
        let data: Data = data_region(1024, 0).into();
        let subdata = data.into_subregion(5..=1000);
        assert_eq!(subdata.as_ref(), &data_region(1024, 0)[5..=1000]);
    }

    #[test]
    fn given_inclusivesubregiondata_when_callingasmut() {
        let data: Data = data_region(1024, 0).into();
        let mut subdata = data.into_subregion(5..=1000);
        assert_eq!(subdata.as_mut(), &data_region(1024, 0)[5..=1000]);
    }

    #[test]
    fn given_inclusivesubregiondata_when_gettingavailablebytes() {
        let data: Data = data_region(1024, 0).into();
        let subdata = data.into_subregion(5..=1000);
        assert_eq!(5, subdata.available_prefix_bytes());
        assert_eq!(23, subdata.available_suffix_bytes());
    }

    #[test]
    fn nested_subregions_still_do_the_right_thing() {
        let data: Data = data_region(1024, 0).into();
        let subdata = data
            .into_subregion(..)
            .into_subregion(5..)
            .into_subregion(..1000)
            .into_subregion(..=950)
            .into_subregion(10..900)
            .into_subregion(3..=800)
            // and all types of ranges again, just in case they don't work if a certain other range happens beforehand
            .into_subregion(..)
            .into_subregion(5..)
            .into_subregion(..700)
            .into_subregion(..=650)
            .into_subregion(10..600)
            .into_subregion(3..=500);
        assert_eq!(36, subdata.available_prefix_bytes());
        assert_eq!(490, subdata.available_suffix_bytes());
        assert_eq!(
            subdata.as_ref(),
            &data_region(1024, 0)[..][5..][..1000][..=950][10..900][3..=800][..][5..][..700]
                [..=650][10..600][3..=500]
        );
    }

    #[test]
    #[should_panic(
        expected = "Range end out of bounds. Tried to access subregion ..=1024 for a Data instance of length 1024"
    )]
    fn given_fullrangedata_when_tryingtogrowendbeyondlength_with_inclusiverange_then_panics() {
        let data: Data = data_region(1024, 0).into();
        data.into_subregion(..=1024);
    }

    #[test]
    #[should_panic(
        expected = "Range end out of bounds. Tried to access subregion ..=100 for a Data instance of length 100"
    )]
    fn given_subrangedata_when_tryingtogrowendbeyondlength_with_inclusiverange_then_panics() {
        let data: Data = data_region(1024, 0).into();
        let data = data.into_subregion(0..100);
        data.into_subregion(..=100);
    }

    #[test]
    #[should_panic(
        expected = "Range end out of bounds. Tried to access subregion ..1025 for a Data instance of length 1024"
    )]
    fn given_fullrangedata_when_tryingtogrowendbeyondlength_with_exclusiverange_then_panics() {
        let data: Data = data_region(1024, 0).into();
        data.into_subregion(..1025);
    }

    #[test]
    #[should_panic(
        expected = "Range end out of bounds. Tried to access subregion ..101 for a Data instance of length 100"
    )]
    fn given_subrangedata_when_tryingtogrowendbeyondlength_with_exclusiverange_then_panics() {
        let data: Data = data_region(1024, 0).into();
        let data = data.into_subregion(0..100);
        data.into_subregion(..101);
    }

    #[test]
    fn given_fullrangedata_when_tryingtogrowstartbeyondend_then_returnszerolengthrange() {
        let data: Data = data_region(1024, 0).into();
        let data = data.into_subregion(5000..400);
        assert_eq!(0, data.len());
    }
}