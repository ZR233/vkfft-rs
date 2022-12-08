cmake_minimum_required(VERSION 3.11)
project(VkFFT)
set(CMAKE_CONFIGURATION_TYPES "Release" CACHE STRING "" FORCE)
set(CMAKE_BUILD_TYPE "Release" CACHE STRING "" FORCE)
set(VKFFT_BACKEND 0 CACHE STRING "0 - Vulkan, 1 - CUDA, 2 - HIP, 3 - OpenCL, 4 - Level Zero, 5 - Metal")

if(${VKFFT_BACKEND} EQUAL 0)
    find_package(Vulkan REQUIRED)
#    include_directories("${Vulkan_INCLUDE_DIR}")
    include_directories("${CMAKE_INSTALL_PREFIX}/include")
    install(DIRECTORY "${Vulkan_INCLUDE_DIR}/vulkan" DESTINATION include)
    install(FILES "${Vulkan_LIBRARIES}/" DESTINATION lib)

#    add_subdirectory(${CMAKE_CURRENT_SOURCE_DIR}/glslang-master)
endif()

set(VK_HOME "${CMAKE_CURRENT_SOURCE_DIR}/../VkFFT-1.2.31")

include_directories("${VK_HOME}/vkFFT")

add_library(half INTERFACE)

add_library(${PROJECT_NAME} STATIC wrapper.cpp)
target_include_directories(${PROJECT_NAME} PUBLIC ${VK_HOME}/half_lib/)
target_include_directories(${PROJECT_NAME} PUBLIC ${CMAKE_INSTALL_PREFIX}/include/glslang/Include/)

if(${VKFFT_BACKEND} EQUAL 0)
#    target_include_directories(${PROJECT_NAME} PUBLIC ${CMAKE_CURRENT_SOURCE_DIR}/glslang-master/glslang/Include/)
    target_link_libraries(${PROJECT_NAME} PUBLIC SPIRV glslang Vulkan::Vulkan half)
endif()

install(TARGETS ${PROJECT_NAME} DESTINATION lib)