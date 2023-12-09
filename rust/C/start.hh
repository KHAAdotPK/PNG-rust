/*
    lib/rust/C/start.hh
    Q@khaa.pk
 */

#ifdef __cplusplus
#include <iostream>
#include "./../../csv/parser.hh"
#endif

#ifndef CLAP_START_HH
#define CLAP_START_HH

#define HELP_STR_START          "("
#define HELP_STR_START_SIZE      1
#define HELP_STR_END            ")"
#define HELP_STR_END_SIZE        1

// Processed Command Line Argument
typedef struct pcla 
{
    int l; // Line number 
    int t; // Token number
    int i; // Index into argv
    int argc; // Number of arguments for this command line argument

    struct pcla *next;
    struct pcla *prev;
} PCLA ;

typedef struct commandLines 
{
    int nTokens;
    size_t helpStrPosition;
    size_t helpStrSize;

    int nClaps;
    PCLA *clap;

    struct commandLines *next;
    struct commandLines *prev;
} COMMANDLINES ;

#ifdef EXPORT_IMPORT
#undef EXPORT_IMPORT
#endif

#ifndef __cplusplus
#define EXPORT_IMPORT __declspec( dllimport )
#else
#define EXPORT_IMPORT __declspec( dllexport )
#endif

#ifdef __cplusplus
extern "C" {  // only need to export C interface if
              // used by C++ source code      
#endif

EXPORT_IMPORT void __stdcall Bubble(PCLA* head);
EXPORT_IMPORT COMMANDLINES* __stdcall StartClapping (int argc, char* argv[], const char *command);
EXPORT_IMPORT void __stdcall StopClapping (COMMANDLINES *lines);
EXPORT_IMPORT void __stdcall BubbleAfter (int argc, char* argv[], const char *command, COMMANDLINES* lines);

EXPORT_IMPORT COMMANDLINES* __stdcall FindArg (COMMANDLINES *lines, const char *command, const char *arg);
EXPORT_IMPORT void HelpTemplate (COMMANDLINES *lines, const char *command);
EXPORT_IMPORT void __stdcall HelpLineTemplate (COMMANDLINES *lines, const char *command, const char *arg);

#ifdef __cplusplus
}
#endif

#ifdef __cplusplus

void __stdcall Stop (PCLA *head);

#endif

#endif