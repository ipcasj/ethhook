# Install script for directory: /Users/igor/rust_projects/capstone0/ethhook-c

# Set the install prefix
if(NOT DEFINED CMAKE_INSTALL_PREFIX)
  set(CMAKE_INSTALL_PREFIX "/usr/local")
endif()
string(REGEX REPLACE "/$" "" CMAKE_INSTALL_PREFIX "${CMAKE_INSTALL_PREFIX}")

# Set the install configuration name.
if(NOT DEFINED CMAKE_INSTALL_CONFIG_NAME)
  if(BUILD_TYPE)
    string(REGEX REPLACE "^[^A-Za-z0-9_]+" ""
           CMAKE_INSTALL_CONFIG_NAME "${BUILD_TYPE}")
  else()
    set(CMAKE_INSTALL_CONFIG_NAME "Debug")
  endif()
  message(STATUS "Install configuration: \"${CMAKE_INSTALL_CONFIG_NAME}\"")
endif()

# Set the component getting installed.
if(NOT CMAKE_INSTALL_COMPONENT)
  if(COMPONENT)
    message(STATUS "Install component: \"${COMPONENT}\"")
    set(CMAKE_INSTALL_COMPONENT "${COMPONENT}")
  else()
    set(CMAKE_INSTALL_COMPONENT)
  endif()
endif()

# Is this installation the result of a crosscompile?
if(NOT DEFINED CMAKE_CROSSCOMPILING)
  set(CMAKE_CROSSCOMPILING "FALSE")
endif()

# Set path to fallback-tool for dependency-resolution.
if(NOT DEFINED CMAKE_OBJDUMP)
  set(CMAKE_OBJDUMP "/usr/bin/objdump")
endif()

if(CMAKE_INSTALL_COMPONENT STREQUAL "Unspecified" OR NOT CMAKE_INSTALL_COMPONENT)
  file(INSTALL DESTINATION "${CMAKE_INSTALL_PREFIX}/bin" TYPE EXECUTABLE FILES "/Users/igor/rust_projects/capstone0/build/ethhook-ingestor")
  if(EXISTS "$ENV{DESTDIR}${CMAKE_INSTALL_PREFIX}/bin/ethhook-ingestor" AND
     NOT IS_SYMLINK "$ENV{DESTDIR}${CMAKE_INSTALL_PREFIX}/bin/ethhook-ingestor")
    execute_process(COMMAND /opt/anaconda3/bin/install_name_tool
      -delete_rpath "/opt/homebrew/Cellar/hiredis/1.3.0/lib"
      -delete_rpath "/opt/homebrew/Cellar/libwebsockets/4.5.2/lib"
      -delete_rpath "/opt/homebrew/Cellar/libevent/2.1.12_1/lib"
      -delete_rpath "/opt/homebrew/Cellar/libmicrohttpd/1.0.2/lib"
      "$ENV{DESTDIR}${CMAKE_INSTALL_PREFIX}/bin/ethhook-ingestor")
    if(CMAKE_INSTALL_DO_STRIP)
      execute_process(COMMAND "/usr/bin/strip" -u -r "$ENV{DESTDIR}${CMAKE_INSTALL_PREFIX}/bin/ethhook-ingestor")
    endif()
  endif()
endif()

if(CMAKE_INSTALL_COMPONENT STREQUAL "Unspecified" OR NOT CMAKE_INSTALL_COMPONENT)
  file(INSTALL DESTINATION "${CMAKE_INSTALL_PREFIX}/bin" TYPE EXECUTABLE FILES "/Users/igor/rust_projects/capstone0/build/ethhook-processor")
  if(EXISTS "$ENV{DESTDIR}${CMAKE_INSTALL_PREFIX}/bin/ethhook-processor" AND
     NOT IS_SYMLINK "$ENV{DESTDIR}${CMAKE_INSTALL_PREFIX}/bin/ethhook-processor")
    execute_process(COMMAND /opt/anaconda3/bin/install_name_tool
      -delete_rpath "/opt/homebrew/Cellar/hiredis/1.3.0/lib"
      -delete_rpath "/opt/homebrew/Cellar/libwebsockets/4.5.2/lib"
      -delete_rpath "/opt/homebrew/Cellar/libevent/2.1.12_1/lib"
      -delete_rpath "/opt/homebrew/Cellar/libmicrohttpd/1.0.2/lib"
      "$ENV{DESTDIR}${CMAKE_INSTALL_PREFIX}/bin/ethhook-processor")
    if(CMAKE_INSTALL_DO_STRIP)
      execute_process(COMMAND "/usr/bin/strip" -u -r "$ENV{DESTDIR}${CMAKE_INSTALL_PREFIX}/bin/ethhook-processor")
    endif()
  endif()
endif()

if(CMAKE_INSTALL_COMPONENT STREQUAL "Unspecified" OR NOT CMAKE_INSTALL_COMPONENT)
  file(INSTALL DESTINATION "${CMAKE_INSTALL_PREFIX}/bin" TYPE EXECUTABLE FILES "/Users/igor/rust_projects/capstone0/build/ethhook-delivery")
  if(EXISTS "$ENV{DESTDIR}${CMAKE_INSTALL_PREFIX}/bin/ethhook-delivery" AND
     NOT IS_SYMLINK "$ENV{DESTDIR}${CMAKE_INSTALL_PREFIX}/bin/ethhook-delivery")
    execute_process(COMMAND /opt/anaconda3/bin/install_name_tool
      -delete_rpath "/opt/homebrew/Cellar/hiredis/1.3.0/lib"
      -delete_rpath "/opt/homebrew/Cellar/libwebsockets/4.5.2/lib"
      -delete_rpath "/opt/homebrew/Cellar/libevent/2.1.12_1/lib"
      -delete_rpath "/opt/homebrew/Cellar/libmicrohttpd/1.0.2/lib"
      "$ENV{DESTDIR}${CMAKE_INSTALL_PREFIX}/bin/ethhook-delivery")
    if(CMAKE_INSTALL_DO_STRIP)
      execute_process(COMMAND "/usr/bin/strip" -u -r "$ENV{DESTDIR}${CMAKE_INSTALL_PREFIX}/bin/ethhook-delivery")
    endif()
  endif()
endif()

if(CMAKE_INSTALL_COMPONENT STREQUAL "Unspecified" OR NOT CMAKE_INSTALL_COMPONENT)
  file(INSTALL DESTINATION "${CMAKE_INSTALL_PREFIX}/bin" TYPE EXECUTABLE FILES "/Users/igor/rust_projects/capstone0/build/ethhook-admin-api")
  if(EXISTS "$ENV{DESTDIR}${CMAKE_INSTALL_PREFIX}/bin/ethhook-admin-api" AND
     NOT IS_SYMLINK "$ENV{DESTDIR}${CMAKE_INSTALL_PREFIX}/bin/ethhook-admin-api")
    execute_process(COMMAND /opt/anaconda3/bin/install_name_tool
      -delete_rpath "/opt/homebrew/Cellar/hiredis/1.3.0/lib"
      -delete_rpath "/opt/homebrew/Cellar/libwebsockets/4.5.2/lib"
      -delete_rpath "/opt/homebrew/Cellar/libevent/2.1.12_1/lib"
      -delete_rpath "/opt/homebrew/Cellar/libmicrohttpd/1.0.2/lib"
      "$ENV{DESTDIR}${CMAKE_INSTALL_PREFIX}/bin/ethhook-admin-api")
    if(CMAKE_INSTALL_DO_STRIP)
      execute_process(COMMAND "/usr/bin/strip" -u -r "$ENV{DESTDIR}${CMAKE_INSTALL_PREFIX}/bin/ethhook-admin-api")
    endif()
  endif()
endif()

string(REPLACE ";" "\n" CMAKE_INSTALL_MANIFEST_CONTENT
       "${CMAKE_INSTALL_MANIFEST_FILES}")
if(CMAKE_INSTALL_LOCAL_ONLY)
  file(WRITE "/Users/igor/rust_projects/capstone0/build/install_local_manifest.txt"
     "${CMAKE_INSTALL_MANIFEST_CONTENT}")
endif()
if(CMAKE_INSTALL_COMPONENT)
  if(CMAKE_INSTALL_COMPONENT MATCHES "^[a-zA-Z0-9_.+-]+$")
    set(CMAKE_INSTALL_MANIFEST "install_manifest_${CMAKE_INSTALL_COMPONENT}.txt")
  else()
    string(MD5 CMAKE_INST_COMP_HASH "${CMAKE_INSTALL_COMPONENT}")
    set(CMAKE_INSTALL_MANIFEST "install_manifest_${CMAKE_INST_COMP_HASH}.txt")
    unset(CMAKE_INST_COMP_HASH)
  endif()
else()
  set(CMAKE_INSTALL_MANIFEST "install_manifest.txt")
endif()

if(NOT CMAKE_INSTALL_LOCAL_ONLY)
  file(WRITE "/Users/igor/rust_projects/capstone0/build/${CMAKE_INSTALL_MANIFEST}"
     "${CMAKE_INSTALL_MANIFEST_CONTENT}")
endif()
