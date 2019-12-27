POSTS = posts
POSTDIRS = 2017 2018 2019 misc
COMPILER = compiler
LINKER = linker

.PHONY: posts
posts: $(COMPILER) $(LINKER)

.PHONY: compiler
$(COMPILER):
	$(MAKE) -C templating/compiler

.PHONY: linker
$(LINKER):
	$(MAKE) -C templating/linker

#  post_path: $(shell find posts -type f)
#  files: $(shell find posts -type f <print basename>)
#  file_dirs: $(shell find posts -type f <print basename && parent dir>
#  dirs: $(shell find posts -type d) ## for compile/link
#  dest_path_target: out/posts/$(file_dirs)
#    - depends on $(post_path) for tracking changes
