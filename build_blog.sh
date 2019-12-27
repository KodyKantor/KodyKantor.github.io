#!/bin/bash
#
# A simple script to run the templating tools.
# This should be part of the Makefile somehow.
#

LINK_ROOT=./link_out
COMP_ROOT=./comp_out
TMPL_DIR=./templating

COMP_BIN=$TMPL_DIR/compiler/target/release/compiler
LINK_BIN=$TMPL_DIR/linker/target/release/linker
POST_DIRS=$(ls posts)

make
mkdir $COMP_ROOT
mkdir $LINK_ROOT

for postdir in $POST_DIRS;
do
	echo compiling $postdir
	mkdir $COMP_ROOT/$postdir
	mkdir $LINK_ROOT/$postdir
	./templating/compiler/target/release/compiler -o $COMP_ROOT/$postdir \
		-i posts/$postdir
	./templating/linker/target/release/linker -t $TMPL_DIR/template.html \
		-c $TMPL_DIR/template.css -o $LINK_ROOT/$postdir \
		-i $COMP_ROOT/$postdir
done
