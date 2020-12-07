## AOC 2020 Day 07

How to solve this without using any allocations, vectors or hashmaps, in about 60us
including parsing the input?

Here's the main ideas of the algorithm:

- First, it would be nice to encode all those "shiny gold" and "striped crimson" words
  into a manageable form. The total number of word pairs would fit within a 2-byte integer,
  so we'll try to encode them as [1, 2, ...].
- Note that in order to fully discriminate a word pair, it is sufficient to only look at the
  first 2 letters of the first word and first 4 letters of the second word. That is, the letters
  of interest in "shiny gold" are "s h g o l d".
- How do we pack those 6 letters efficiently into a single number? That makes up for 6 bytes,
  so a single lookup table won't do the trick, and we don't want to use a hashmap. Here's one
  way to do it:
  - For each pair of the 6 letters, reinterpret them as 2-byte integers (in native
    endianness, in order to avoid byteswaps), so we essentially have triple `[u16; 3]` 
    that we'd like to convert to a `u16` that we'll call *word-pair ID*.
  - Create three tables, `[u8; 65536]` each (64 KB), that would serve as maps and fill them with
    `0xff` initially; each table also holds the number of different letter-pairs it has
    seen so far. Upon a lookup, we check the corresponding cell in the table: if it's non-empty,
    we return the number within; if it's empty, we write the current count into that cell and
    increase the count. (Note: we're using the fact that there's no more than 256 distinct
    values for letter pairs in the input data; if this wasn't the case, we'd just double the
    table size by using two-byte values).
  - Using the three 2-byte tables, we can very quickly encode the three letter pairs into
    three `u8` integers. Next step - how do we collapse them into a single `u16`? Same story,
    we observe that each of these integers only has up to 32 distinct values, so we build
    a second-level table, `[u16; 1 << 15]`, and repeat the above approach. The key that we'll
    use to access this table will be the three integers bit-stacked together, making it for
    the total of 15 bits. (Note: if there were more than 32 distinct values, we'd either
    have to use a wider table, or do a three-level lookup instead).
  - To sum it up... using four lookup tables in total, we can quickly convert 3 letter-pairs
    into a single `u16` number, without any allocations, loops etc. 
  - One minor detail: IDs start from 1 and not 0, to simplify some of the logic below.
- Now we can write a byte-by-byte parser that extracts the above letter pairs from the raw
  input string and builds the graph structure. It's quite trivial, the main point from
  performance standpoint is to not parse what we don't need (skip as far as possible with
  `memchr`, and only parse those 6 letters we need from each word pair), and it can be done
  quite succinctly and, again, without any allocations.
- On to the graph structure - the idea is the same for part 1 and part 2, with minor differences
  in the details:
  - We'll use `[[u16; 32]; 1024]` matrix to represent the graph (that is, a 1024x32 matrix
    of 2-byte integers). It's more than sufficient to represent the input data.
  - Each row corresponds to a single word-pair ID generated using the lookup algorithm above.
  - Column 0 contains the number of elements in the row (N).
  - Part 1: cells in row R starting from column 1 contain word-pair IDs of parents of element R
    (i.e., "which bags can this bag be contained in directly").
  - Part 2: cells in row R starting from column 1 contain 2-tuples (word-pair ID, count)
    corresponding to children of R (that is, all odd indices starting from 1 will contain the 
    IDs and all even indices starting from 2 will contain the corresponding counts obtained from
    the input data).
  - Since word-pair IDs start with 1, the first row is just zeros.
- Once the graph is parsed, actually traversing it is trivial and takes almost no time 
  (about 1us for part 1, about 0.2us for part 2). You just have to recursively jump around
  the rows and fold the output.
