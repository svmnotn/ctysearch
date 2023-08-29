#ifndef EXAMPLE_H
#define EXAMPLE_H

uint8_t* test(uint8_t x);
void test1(uint8_t a, uint8_t *a);
Test_t test2(uint8_t a, uint8_t *a);
Test_t *test3(Test_t a, uint8_t *a);

typedef uint8_t* (*test4_t)(uint8_t x);
typedef void (*test5_t)(uint8_t a, uint8_t *a);
typedef Test_t (*test6_t)(uint8_t a, uint8_t *a);
typedef Test_t *(*test7_t)(Test_t a, uint8_t *a);

struct I_Test_Api_t;

typedef struct {
    const struct I_Test_Api_t *api;
} I_Test_t;

typedef struct I_Test_Api_t {
    void (*test8)(I_Test_t *instance, void *ctx);
    I_Test_t *(*test9)(I_Test_t *instance);
} I_Test_Api_t;

#endif
