SUB_DIRS=origin hello_c fileops_c

all: $(SUB_DIRS)

$(SUB_DIRS): mapfile_c FORCE
	make -C $@

FORCE:

.PHONY: all SUB_DIRS FORCE
