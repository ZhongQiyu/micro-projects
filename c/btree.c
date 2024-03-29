#include <stdio.h>
#include <stdlib.h> //malloc, free
#include "btree.h" //tnode

//OK
tnode *btree_create() {
  tnode *tree = NULL; 
  tree = malloc(sizeof(tnode));
  return tree;
}

//OK
tnode *create_new_node(int v) {
  tnode* node;
  node = btree_create();
  node -> element = v;
  node -> right = NULL;
  node -> left = NULL;
  return node;
}

//OK
void btree_insert(int v, tnode *t) {
  tnode *node = t;

  if (v > node -> element) {
    if (node -> right == NULL) {
      tnode* new_node = create_new_node(v);
      node -> right = new_node;
    } else {
      btree_insert(v, node -> right);
    }
  } else if (v <= node -> element) {
    if (node -> left == NULL) {
      tnode* new_node = create_new_node(v);
      node -> left = new_node;
    } else {
      btree_insert(v, node -> left);
    }  } else {
  } 
}

void btree_dump(tnode* t) {
  // if (t !)
  tnode* current = t;
  // printf("%d\n",current -> element);
  if (current -> left != NULL) {
    btree_dump(current -> left);
  }
  printf("%d\n", current -> element);
  if (current -> right != NULL) {
    btree_dump(current -> right);
  }
}

//OK
void btree_destroy(tnode *t) {
  free(t);
}