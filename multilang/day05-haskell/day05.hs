#!/usr/bin/env runhaskell

import System.IO (readFile)
import Data.Char (isSpace)

parseChar :: Char -> Int -> Int
parseChar c n
    | n >= 3 = fromEnum (c == 'B')
    | otherwise = fromEnum (c == 'R')

parseId :: String -> Int
parseId [] = 0
parseId (c:s) =
    let n = length s
    in (parseChar c n) * (2 ^ n) + parseId s

part1 :: [Int] -> Int
part1 ids = maximum ids

part2 :: [Int] -> Int
part2 ids =
    let a = minimum ids
        b = maximum ids
        s = sum ids
        t = ((b - a + 1) * (a + b)) `div` 2
    in t - s 

main = do  
    contents <- readFile "input.txt"  
    let ids = map parseId $ filter (not . all isSpace) $ lines contents
    print $ part1 ids
    print $ part2 ids
