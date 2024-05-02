typedef struct Tnode tnode;
typedef struct Tnode {
  int element;
  tnode *right;
  tnode *left; 
} tnode;

tnode *btree_create();
void btree_insert(int v, tnode *t);
void btree_destroy(tnode *t);
void btree_dump(tnode *t);
