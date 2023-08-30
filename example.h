#ifndef EXAMPLE_H
#define EXAMPLE_H

uint8_t* test(uint8_t x);
void test1(uint8_t a, uint8_t *a);
Test_t test2(uint8_t a, uint8_t *a);
Test_t *test3(Test_t a, uint8_t *a);
struct Test_t *test10(struct Test_t a, uint8_t *a);
enum Test_t *test13(enum Test_t a, uint8_t *a);
void test14();
void test15(void);

typedef uint8_t* (*test4_t)(uint8_t x);
typedef void (*test5_t)(uint8_t a, uint8_t *a);
typedef Test_t (*test6_t)(uint8_t a, uint8_t *a);
typedef Test_t *(*test7_t)(Test_t a, uint8_t *a);
typedef struct Test_t *(*test11_t)(struct Test_t a, uint8_t *a);

typedef struct {
    void (*test8)(Test_t *instance, void *ctx);
    Test_t *(*test9)(Test_t *instance);
    struct Test_t *(*test12)(struct Test_t *instance);
} Test_t;

#endif
