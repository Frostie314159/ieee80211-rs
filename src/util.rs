use alloc::{vec, vec::Vec};
use scroll::{
    ctx::{MeasureWith, TryIntoCtx},
    Pwrite,
};

pub fn write_to_vec<Ctx, T: TryIntoCtx + MeasureWith<Ctx>>(
    data: T,
    ctx: &Ctx,
) -> Result<Vec<u8>, scroll::Error>
where
    <T as TryIntoCtx>::Error: From<scroll::Error>,
    scroll::Error: From<<T as TryIntoCtx>::Error>,
{
    let mut buffer = vec![0x00; data.measure_with(ctx)];
    buffer.as_mut_slice().pwrite(data, 0)?;
    Ok(buffer)
}
