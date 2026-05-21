#ifndef MECAB_RUST_CONFIG_H
#define MECAB_RUST_CONFIG_H

#if defined(__GNUC__) || defined(__clang__)
    #define HAVE_GCC_ATOMIC_OPS 1
#endif

#if defined(__APPLE__)
    #define HAVE_OSX_ATOMIC_OPS 1
#endif

#ifndef _WIN32
    /* Define to 1 if you have the <ctype.h> header file. */
    #define HAVE_CTYPE_H 1
    
    /* Define to 1 if you have the <dirent.h> header file. */
    #define HAVE_DIRENT_H 1
    
    /* Define to 1 if you have the <dlfcn.h> header file. */
    #define HAVE_DLFCN_H 1
    
    /* Define to 1 if you have the <fcntl.h> header file. */
    #define HAVE_FCNTL_H 1
    
    /* Define to 1 if you have the `getenv' function. */
    #define HAVE_GETENV 1
    
    /* Define to 1 if you have the `getpagesize' function. */
    #define HAVE_GETPAGESIZE 1
    
    /* Define if you have the iconv() function and it works. */
    #define HAVE_ICONV 1
    
    /* Define to 1 if you have the <inttypes.h> header file. */
    #define HAVE_INTTYPES_H 1
    
    /* Define to 1 if you have the <io.h> header file. */
    /* #undef HAVE_IO_H */
    
    /* Define to 1 if you have the `pthread' library (-lpthread). */
    #define HAVE_LIBPTHREAD 1
    
    /* Define to 1 if you have the <memory.h> header file. */
    #define HAVE_MEMORY_H 1
    
    /* Define to 1 if you have a working `mmap' system call. */
    #define HAVE_MMAP 1
    
    /* Define to 1 if you have the `opendir' function. */
    #define HAVE_OPENDIR 1
    
    /* Define to 1 if you have the <pthread.h> header file. */
    #define HAVE_PTHREAD_H 1
    
    /* Define to 1 if you have the <stdint.h> header file. */
    #define HAVE_STDINT_H 1
    
    /* Define to 1 if you have the <stdlib.h> header file. */
    #define HAVE_STDLIB_H 1
    
    /* Define to 1 if you have the <strings.h> header file. */
    #define HAVE_STRINGS_H 1
    
    /* Define to 1 if you have the <string.h> header file. */
    #define HAVE_STRING_H 1
    
    /* Define to 1 if you have the <sys/mman.h> header file. */
    #define HAVE_SYS_MMAN_H 1
    
    /* Define to 1 if you have the <sys/param.h> header file. */
    #define HAVE_SYS_PARAM_H 1
    
    /* Define to 1 if you have the <sys/stat.h> header file. */
    #define HAVE_SYS_STAT_H 1
    
    /* Define to 1 if you have the <sys/times.h> header file. */
    #define HAVE_SYS_TIMES_H 1
    
    /* Define to 1 if you have the <sys/types.h> header file. */
    #define HAVE_SYS_TYPES_H 1
    
    /* */
    #define HAVE_TLS_KEYWORD 1
    
    /* Define to 1 if you have the <unistd.h> header file. */
    #define HAVE_UNISTD_H 1
    
    /* Define to 1 if the system has the type `unsigned long long int'. */
    #define HAVE_UNSIGNED_LONG_LONG_INT 1
    
    /* Define as const if the declaration of iconv() needs const. */
    #define ICONV_CONST 
#endif

/* Name of package */
#define PACKAGE "mecab"

/* Define to the address where bug reports for this package should be sent. */
#define PACKAGE_BUGREPORT ""

/* Define to the full name of this package. */
#define PACKAGE_NAME ""

/* Define to the full name and version of this package. */
#define PACKAGE_STRING ""

/* Define to the one symbol short name of this package. */
#define PACKAGE_TARNAME ""

/* Define to the home page for this package. */
#define PACKAGE_URL ""

/* Define to the version of this package. */
#define PACKAGE_VERSION ""

/* The size of `char', as computed by sizeof. */
#define SIZEOF_CHAR 1

/* The size of `int', as computed by sizeof. */
#define SIZEOF_INT 4

/* The size of `long', as computed by sizeof. */
#define SIZEOF_LONG 8

/* The size of `long long', as computed by sizeof. */
#define SIZEOF_LONG_LONG 8

/* The size of `short', as computed by sizeof. */
#define SIZEOF_SHORT 2

/* The size of `size_t', as computed by sizeof. */
#define SIZEOF_SIZE_T 8

/* Define to 1 if you have the ANSI C header files. */
#define STDC_HEADERS 1

/* Version number of package */
#define VERSION "0.996"

#endif /* MECAB_RUST_CONFIG_H */
