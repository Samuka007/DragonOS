#pragma once

#include "fat32.h"
#include <filesystem/VFS/VFS.h>

/**
 * @brief 请求分配指定数量的簇
 * 
 * @param inode 要分配簇的inode
 * @param clusters 返回的被分配的簇的簇号结构体
 * @param num_clusters 要分配的簇的数量
 * @return int 错误码
 */
int fat32_alloc_clusters(struct vfs_index_node_t *inode, uint32_t *clusters, int32_t num_clusters);

/**
 * @brief 释放从属于inode的，从cluster开始的所有簇
 * 
 * @param inode 指定的文件的inode
 * @param cluster 指定簇
 * @return int 错误码
 */
int fat32_free_clusters(struct vfs_index_node_t * inode, int32_t cluster);

/**
 * @brief 读取指定簇的FAT表项
 *
 * @param fsbi fat32超级块私有信息结构体
 * @param cluster 指定簇
 * @return uint32_t 下一个簇的簇号
 */
uint32_t fat32_read_FAT_entry(fat32_sb_info_t *fsbi, uint32_t cluster);

/**
 * @brief 写入指定簇的FAT表项
 *
 * @param fsbi fat32超级块私有信息结构体
 * @param cluster 指定簇
 * @param value 要写入该fat表项的值
 * @return uint32_t errcode
 */
uint32_t fat32_write_FAT_entry(fat32_sb_info_t *fsbi, uint32_t cluster, uint32_t value);

/**
 * @brief 在父亲inode的目录项簇中，寻找连续num个空的目录项
 *
 * @param parent_inode 父inode
 * @param num 请求的目录项数量
 * @param mode 操作模式
 * @param res_sector 返回信息：缓冲区对应的扇区号
 * @param res_cluster 返回信息：缓冲区对应的簇号
 * @param res_data_buf_base 返回信息：缓冲区的内存基地址（记得要释放缓冲区内存！！！！）
 * @return struct fat32_Directory_t* 符合要求的entry的指针（指向地址高处的空目录项，也就是说，有连续num个≤这个指针的空目录项）
 */
struct fat32_Directory_t *fat32_find_empty_dentry(struct vfs_index_node_t *parent_inode, uint32_t num, uint32_t mode, uint32_t *res_sector, uint64_t *res_cluster, uint64_t *res_data_buf_base);