/// Inserts a slice into a vector at a given offset, shifting elements 
/// starting at the offset towards the end.
/// Returns 0 (zero) if the operation was successful, non-zero if the offset 
/// is larger than the current length of the destination vector. In the latter 
/// case, everything stays untouched.
pub(crate) fn insert_multiple_at<T>
(
    vec_dst: &mut Vec<T>,
    offset:  usize,
    vec_src: &mut Vec<T>,
)
-> usize
where T: Copy 
{
    match (vec_dst.len(), vec_src.len()) 
    {
        (_, 0)           => 0,
        (current_len, _) => {

            // If this is true we return at this point as this would cause a
            // "gap" between existing and new vector contents
            if current_len < offset
            {
                return std::cmp::max(1, current_len);
            }

            // Reserve without over-allocation space needed for new elements
            vec_dst.reserve_exact(vec_src.len());

            let mut temp = vec_dst.split_off(offset);
            vec_dst.append(vec_src);
            vec_dst.append(&mut temp);

            return 0;
        },
    }
}

/*
/// Inserts a slice into a vector at a given offset, shifting elements 
/// starting at the offset towards the end.
/// Returns 0 (zero) if the operation was successful, non-zero if the offset 
/// is larger than the current length of the destination vector. In the latter 
/// case, everything stays untouched.
pub(crate) fn insert_multiple_at<T>
(
    vec_dst: &mut Vec<T>, 
    offset:  usize, 
    vec_src: &mut [T]
)
-> usize
where T: Copy 
{
    match (vec_dst.len(), vec_src.len()) 
    {
        (_, 0)           => 0,
        (current_len, _) => {

            // Elements that need to be moved to make way for the new ones
            let move_count = current_len - offset;

            // If this is less than 0 we return at this point as this would
            // cause a "gap" between existing and new vector contents
            // (move_count is usize and thus can't be less than 0)
            if current_len < offset
            {
                return std::cmp::max(1, current_len);
            }

            // Reserve without over-allocation space needed for new elements
            vec_dst.reserve_exact(vec_src.len());

            unsafe
            {
                // Pointer to the first location where vec_src elements will
                // be placed. 
                // Called `src` at this stage as previously it has to serve
                // as source for elements that require to be copied to the
                // right to make way 
                let src = vec_dst.as_mut_ptr().offset(offset as isize);

                // Set the new length of the vector after the operation
                vec_dst.set_len(current_len + vec_src.len());

                // Check if there are any elements that require to be moved
                if move_count > 0 
                {
                    let dst = src.offset(vec_src.len() as isize);
                    std::ptr::copy(
                        src,                                                    // Source pointer
                        dst,                                                    // Destination pointer
                        move_count                                              // How many elements to copy
                    );
                }

                // Copy the new elements at the new "free" locations
                // The previous source pointer `src` becomes the destination of
                // the new elements to be inserted
                // In contrast to the previous copy we can here be sure that
                // the source and destination don't overlap
                std::ptr::copy_nonoverlapping(
                    vec_src.as_mut_ptr(),                                       // Source pointer
                    src,                                                        // Destination pointer
                    vec_src.len()                                               // How many elements to copy (here: ALL)
                );
            }

            return 0;
        },
    }
}
*/