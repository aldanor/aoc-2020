#!/usr/bin/env racket
#lang racket

(define (parse-input input) (string-split input "\n\n" #:trim? #t))

(define (parse-group op group)
    (define persons (string-split group "\n" #:trim? #t))
    (define charsets (map list->set (map string->list persons)))
    (set-count (foldl op (first charsets) charsets)))

(define (compute-sum op input)
    (foldr + 0 (map (curry parse-group op) (parse-input input))))

(define (part1 input) (compute-sum set-union input))
(define (part2 input) (compute-sum set-intersect input))

(define input (file->string "input.txt"))
(part1 input)
(part2 input)
