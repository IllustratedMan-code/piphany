# process!

returns a process derivation

```scheme
(process!
	name : "first-process"
	container : "ubuntu:latest"
	script : #<<''
	   echo "Hi world" > {{out}}/result.txt
	''
)
```

# file!

turn a filepath into a derivation

# df::

Dataframe operations

## df::subset!

works like R's subset

## df::with-column

set column to list of values

## df::read-csv

read a csv into a df

## df::read-excel

Read an excel file into a df, should take sheet name, or return hashmap of dfs

## df::df!

Create a df

## df::as-csv

returns a derivation pointing to a csv file.

## df::as-database

takes either a dataframe or a hashmap of dataframes, returns a derivation pointing to a database file, including excel/sql as a format

# expand!

returns a lazy generator of derivations (not known at compile time) which takes a glob path as input and returns an iterator object. A process can only take one expand! object or family of expand! objects returned by cross! or zip!.

## cross!

returns a function which takes an argument representing either the left or the right of a list containing two elements. At runtime, the iterator would run each element pair (cartesian product)

## zip!

returns a function which takes an argument representing either the left or the right of a list containing two elements. At runtime, the iterator would run each element pair (zipped pairs)

## collapse!

collapses an iterator into a single derivation again

# output!

```scheme
(output!
 "results/proc2-result.txt" : proc2
 "results/proc1" : proc1
 )
```
