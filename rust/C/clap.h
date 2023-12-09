/*
    lib/rust/C/main.hh
    Q@khaa.pk
 */

//#include <stdio.h>
#include <limits.h>

#include "start.hh"

#ifndef CLAP_HH
#define CLAP_HH

#ifdef EXPORT_IMPORT
#undef EXPORT_IMPORT
#endif

#ifndef __cplusplus
#define EXPORT_IMPORT __declspec( dllexport )
#else
#define EXPORT_IMPORT __declspec( dllimport )
#endif

#ifdef __cplusplus
extern "C" {  // only need to export C interface if
              // used by C++ source code      
#endif

EXPORT_IMPORT COMMANDLINES* __stdcall Clap (int argc, char* argv[], const char *command); 
EXPORT_IMPORT void __stdcall Stop (COMMANDLINES *head);

EXPORT_IMPORT COMMANDLINES* __stdcall Find (COMMANDLINES *head, const char *command, const char *arg);
EXPORT_IMPORT void __stdcall Help (COMMANDLINES *head, const char *command);
EXPORT_IMPORT void __stdcall HelpLine (COMMANDLINES *head, const char *command, const char *arg);

EXPORT_IMPORT int __stdcall Common (COMMANDLINES *head);

#ifdef __cplusplus
}
#endif

#ifdef __cplusplus

#endif

#endif