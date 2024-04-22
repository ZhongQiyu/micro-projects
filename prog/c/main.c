#include <stdio.h>
#include <errno.h> //perror, errno
#include <stdlib.h> //srand, rand
#include <time.h> //time
#include <pthread.h> //pthread_exit, pthread_create
#include "btree.h"

tnode *tree;

int error_handling(char* msg) {
  perror(msg);
  return errno;
}

void *func_inserts(void *arg) {
  for (int j = 0; j < 100; j++) {
    // int num = rand()%100;
    // num %= 100000
    // printf("%d\n", num);
    // btree_insert(num, tree);
    btree_insert(rand()%10000, tree);
  }
  return 0;
}

int main() {
  tree = btree_create();
  srand((unsigned)time(NULL));

  pthread_t threads[100];

  for (int i = 0; i < 100; i++) {
    int temp = i;
    if (pthread_create(&threads[i], NULL, *func_inserts, &temp) != 0) {
      error_handling("pthread_create");
    }
  }

  // printf("Start dump\n");
  btree_dump(tree);
  btree_destroy(tree);
  return 0;
}
