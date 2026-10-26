#define _DEFAULT_SOURCE
#include "log_engine.h"

#include <fcntl.h>
#include <stdio.h>
#include <sys/mman.h>
#include <sys/stat.h>
#include <unistd.h>

int mmap_open(const char* filepath, MappedFile* out_file) {
    if (!filepath || !out_file) {
        return -1;
    }

    out_file->data = NULL;
    out_file->size = 0;
    out_file->fd = -1;

    int fd = open(filepath, O_RDONLY);
    if (fd < 0) {
        perror("mmap_open: open failed");
        return -1;
    }

    struct stat sb;
    if (fstat(fd, &sb) < 0) {
        perror("mmap_open: fstat failed");
        close(fd);
        return -1;
    }

    if (sb.st_size == 0) {
        close(fd);
        return -1;
    }

    void* mapped = mmap(NULL, (size_t)sb.st_size, PROT_READ, MAP_PRIVATE, fd, 0);
    if (mapped == MAP_FAILED) {
        perror("mmap_open: mmap failed");
        close(fd);
        return -1;
    }

    /* Hint to the OS kernel that we will scan this file sequentially */
    madvise(mapped, (size_t)sb.st_size, MADV_SEQUENTIAL);

    out_file->data = (const char*)mapped;
    out_file->size = (size_t)sb.st_size;
    out_file->fd = fd;
    return 0;
}

void mmap_close(MappedFile* file) {
    if (!file) {
        return;
    }
    if (file->data && file->data != MAP_FAILED && file->size > 0) {
        munmap((void*)file->data, file->size);
    }
    if (file->fd >= 0) {
        close(file->fd);
    }
    file->data = NULL;
    file->size = 0;
    file->fd = -1;
}
