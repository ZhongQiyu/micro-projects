unsafe_btree: unsafe_btree_create
	./unsafe_btree
unsafe_btree_create: main.c btree.c btree.h
	gcc -Wall -pthread -o unsafe_btree main.c btree.c
clean:
	rm -f *.o unsafe_btree
