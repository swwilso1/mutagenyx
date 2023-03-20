workspace "Mutagenyx" "Mutagenyx architecture documentation" {

    model {
        mutagenyxUser = person "Mutagenyx User" "A person using Mutagenyx to mutate programs" "User"

        mutagenyx = softwareSystem "Mutagenyx" "An application and framework for mutating programs at the AST level." {
         
            group "Mutagenyx Binary" {
                commandLineInterface = container "Command-Line Interface" "Provides the command-line parameters and parsing needed to run Mutagenyx" "Function" "Function"
                algorithmDescriber = container "Algorithm Description Module" "Outputs lists or descriptions of mutation algorithms" "Function" "Function"
                mutationGenerator = container "Mutation Generator" "Manages generating mutations for multiple languages" "Function" "Function"
                filePrettyPrinter = container "File Pretty Printer" "Pretty-prints source or AST files using Mutagenyx language pretty-printers" "Function" "Function"
            }

            group "Mutagenyx Library" {
                mutationAlgorithms = container "Mutation Algorithm Descriptions" "List of mutation algorithms with descriptions" "Struct List" 
                astTraverser = container "ASTTraverser" "Type providing traversal algorithm for any AST that conforms to SimpleAST" "Struct"
                commenter = container "Commenter<AST>" "Trait defining functionality for an object that wants to insert a comment in an AST node" "Trait"
                commenterFactory = container "CommenterFactory<AST>" "Trait defining functionality for an object that can produce Commenter<AST> objects for a specific AST" "Trait" "Factory"
                idMaker = container "Id<AST>" "Trait defining functionality for an object that can convert an AST node to a unique id" "Trait"
                jsonCommentInserter = container "JSONCommentInserter" "Struct providing faclitites to insert comments in to JSON encoded AST" "Struct"
                jsonLanguageDelegate = container "JsonLanguageDelegate" "Implementation of language specific delegate for JSON ASTs" "Struct"
                mutableLanguage = container "MutableLanguage" "Trait defining functionality needed to support a language for mutation" "Trait"
                languageInterface = container "LanguageInterface" "Struct providing capability of loading MutableLanguage trait objects for each supported language" "Struct"
                pathVisitor = container "PathVisitor<AST>" "Struct that can visit nodes in an AST and calculate node paths" "Struct"
                mutableNodesCounter = container "MutableNodesCounter<AST>" "Struct that can visit nodes in an AST and calculate mutable nodes" "Struct"
                mutationMaker = container "MutationMaker<AST>" "Struct that can visit nodes in an AST and mutate the nodes" "Struct"
                mutator = container "Mutator<AST>" "Trait defining functionality needed to implement a mutation algorithm for an AST" "Trait"
                mutatorFactory = container "MutatorFactory<AST>" "Trait defining functionality needed by objects that generator Mutators for a language" "Trait" "Factory"
                namer = container "Namer<AST>" "Trait defining functionality needed by objects that can generate a name string for an AST node" "Trait"
                nodeFinder = container "NodeFinder<AST>" "Trait defining functionality needed by objects that can locate an node with an id" "Trait"
                nodeFinderFactory = container "NodeFinderFactory<AST>" "Trait defining functionality need by objects that can create NodeFinder objects for nodes" "Trait" "Factory"
                nodePrinter = container "NodePrinter<AST>" "Trait defining functionality needed by objects that can pretty-print a node" "Trait"
                nodePrinterFactory = container "NodePrinterFactory<AST>" "Trait defining functionality needed by objects that can create NodePrinter<AST> objects for a language" "Trait" "Factory"
                permissions = container "Permissions" "Struct that supports querying and granting permissions to perform actions on nodes" "Struct"
                permitter = container "Permit<AST>" "Trait defining functionality needed by objectes that can grant permissions to perform actions on nodes" "Trait"
                prettyPrintVisitor = container "PrettyPrintVisitor<AST>" "Struct that can visit nodes in an AST for pretty-printing" "Struct"
                prettyPrinter = container "PrettyPrinter" "Struct that manages low level structured output to a file/stream" "Struct"
                recognizer = container "Recognizer" "Struct that can recognize the file type and language in an input file" "Struct"
            }
        }

    	mutagenyxUser -> mutagenyx "Uses" "" "link"

        algorithmDescriber -> mutationAlgorithms "loads" "" "link"
        astTraverser -> mutableNodesCounter "counts nodes with" "" "link"
        astTraverser -> mutationMaker "mutates nodes with" "" "link"
        astTraverser -> pathVisitor "constructs paths to nodes with" "" "link"
        astTraverser -> prettyPrintVisitor "converts to source code with" "" "link"
        commandLineInterface -> algorithmDescriber "describes algorithms with" "" "link"
        commandLineInterface -> filePrettyPrinter "pretty-prints files with" "" "link"
        commandLineInterface -> mutationGenerator "generates mutations with" "" "link"
        filePrettyPrinter -> languageInterface "loads mutable language with" "" "link"
        filePrettyPrinter -> mutableLanguage "pretty-prints files with" "" "link"
        filePrettyPrinter -> recognizer "recognizes input files with" "" "link"
        jsonCommentInserter -> commenter "inserts comments into AST with" "" "link"
        jsonCommentInserter -> commenterFactory "loads node commenters with" "" "link"
        jsonLanguageDelegate -> jsonCommentInserter "uses to insert comments" "" "link"
        jsonCommentInserter -> nodeFinder "finds nodes that should contain comments with" "" "link"
        jsonCommentInserter -> nodeFinderFactory "loads node finders with" "" "link"
        mutableLanguage -> astTraverser "traverses ASTs with" "" "link"
        mutableLanguage -> jsonLanguageDelegate "interfaces with JSON subsystems with" "" "link"
        mutableLanguage -> mutableNodesCounter "counts nodes with" "" "link"
        mutableLanguage -> mutationMaker "gets comment nodes from" "" "link"
        mutableLanguage -> mutatorFactory "loads mutators with" "" "link"
        mutableNodesCounter -> mutator "checks nodes for mutability with" "" "link"
        mutableNodesCounter -> namer "gets node name with" "" "link"
        mutableNodesCounter -> permitter "gets permission to visit node with" "" "link"
        mutationGenerator -> filePrettyPrinter "pretty-prints AST with" "" "link"
        mutationGenerator -> languageInterface "loads mutable language with" "" "link"
        mutationGenerator -> mutableLanguage "mutates AST with" "" "link"
        mutationGenerator -> recognizer "recognizes input files with" "" "link"
        mutationMaker -> astTraverser "traverses sub-nodes of ASTs with" "" "link"
        mutationMaker -> mutator "mutates AST nodes with" "" "link"
        mutationMaker -> namer "gets node name with" "" "link"
        mutationMaker -> permitter "gets permission to visit node with" "" "link"
        nodeFinder -> idMaker "get node id from" "" "link"
        nodePrinter -> prettyPrinter "pretty-prints AST node with" "" "link"
        pathVisitor -> idMaker "gets node id with" "" "link"
        pathVisitor -> namer "gets node name with" "" "link"
        pathVisitor -> permitter "gets permission to visit node with" "" "link"
        permitter -> permissions "asks permission from" "" "link"
        prettyPrintVisitor -> nodePrinter "uses to pretty-print a node" "" "link"
        prettyPrintVisitor -> nodePrinterFactory "loads a node printer from" "" "link"
    }

    views {

        systemContext mutagenyx "MutagenyxBasicUsage" {
            include *
            autoLayout
        }

        container mutagenyx "Containers" {
            include *
            autoLayout
        }

        dynamic mutagenyx "GenerateMutations" "Generic Mutation Algorithm" {
            commandLineInterface -> mutationGenerator "Sends command-line options to"
            mutationGenerator -> recognizer "recognize input file with"
            mutationGenerator -> languageInterface "load language trait object from"
            mutationGenerator -> mutableLanguage "mutate input file with"
            mutationGenerator -> filePrettyPrinter "pretty print mutated AST with"
            autoLayout
        }

        dynamic mutagenyx "SelectingMutators" "Selecting Mutatable Nodes & Algorithms" {
            commandLineInterface -> mutationGenerator "user specifies algorithms to"
            mutationGenerator -> languageInterface "load language object with"
            mutationGenerator -> mutableLanguage "mutable node count and algorithms with"
            mutableLanguage -> astTraverser "traverse AST with"
            astTraverser -> mutableNodesCounter "count nodes with"
            mutableNodesCounter -> namer "gets node name with"
            mutableNodesCounter -> permitter "asks permission to visit node from"
            mutableNodesCounter -> mutator "check mutability with"
            autoLayout
        }

        dynamic mutagenyx "MutateFile" "How to Make a Mutation" {
            commandLineInterface -> mutationGenerator "user specifies mutation algorithms to"
            mutationGenerator -> languageInterface "load language object with"
            mutationGenerator -> mutableLanguage "mutate input file with"
            mutableLanguage -> mutatorFactory "get mutator from"
            mutableLanguage -> astTraverser "traverse AST with"
            astTraverser -> mutationMaker "visit nodes with"
            mutationMaker -> namer "gets node name with"
            mutationMaker -> permitter "asks permission to visit node from"
            mutationMaker -> mutator "mutate with"
            autoLayout
        }

        dynamic mutagenyx "InsertComment" "Insert Comment Nodes" {
            commandLineInterface -> mutationGenerator "user requests mutation(s) from"
            mutationGenerator -> languageInterface "load language object with"
            mutationGenerator -> mutableLanguage "calculate node paths with"
            mutableLanguage -> astTraverser "traverse AST with"
            astTraverser -> pathVisitor "visit nodes with"
            pathVisitor -> idMaker "get node id from"
            mutationGenerator -> mutableLanguage "ast and node paths to"
            mutableLanguage -> astTraverser "traverse AST with"
            astTraverser -> mutationMaker "visit nodes with"
            mutationMaker -> mutator "mutate with"
            mutableLanguage -> mutationMaker "get comment node from"
            mutableLanguage -> jsonLanguageDelegate "insert comment node with path with"
            jsonLanguageDelegate -> jsonCommentInserter "insert node with path with"
            jsonCommentInserter -> nodeFinderFactory "get node finder from"
            jsonCommentInserter -> nodeFinder "find node with"
            nodeFinder -> idMaker "get node id from"
            jsonCommentInserter -> commenterFactory "get comment inserter from"
            jsonCommentInserter -> commenter "insert comment node"
            autoLayout tb
        }

        dynamic mutagenyx "PrettyPrint" "Pretty Print File" {
            commandLineInterface -> filePrettyPrinter "user requests pretty print from"
            filePrettyPrinter -> recognizer "recognize input file with"
            filePrettyPrinter -> languageInterface "load language object with"
            filePrettyPrinter -> mutableLanguage "pretty-print AST with"
            mutableLanguage -> astTraverser "traverse AST with"
            astTraverser -> prettyPrintVisitor "visit nodes with"
            prettyPrintVisitor -> nodePrinterFactory "node printer from"
            prettyPrintVisitor -> nodePrinter "print node with"
            nodePrinter -> prettyPrinter "print node details with"
            autoLayout
        }

        # theme default

        styles {
            element "Person" {
                color #ffffff
                fontSize 22
                shape Person
            }
            element "User" {
                background #08427b
            }
            element "Software System" {
                background #1168bd
                color #ffffff
            }
            element "Container" {
                background #438dd5
                color #ffffff
            }
            element "Function" {
                background #85bbf0
                color #000000
                fontSize 24
                shape Ellipse
            }
            element "Factory" {
                background #999977
                color #000000
                shape Folder
            }

            relationship "link" {
                thickness 2
                color #a62a17
                style solid
            }
        }
    }
}
