glslangValidator.exe -V .\planetgen.vert -o planetgen.vert.spv
glslangValidator.exe -V .\planetgen.frag -o planetgen.frag.spv

glslangValidator.exe -V ..\..\src\modes\playgame\render\passes\terrain_pass\terrain.frag -o ..\..\src\modes\playgame\render\passes\terrain_pass\terrain.frag.spv
glslangValidator.exe -V ..\..\src\modes\playgame\render\passes\terrain_pass\terrain.vert -o ..\..\src\modes\playgame\render\passes\terrain_pass\terrain.vert.spv

glslangValidator.exe -V ..\..\src\modes\playgame\render\passes\model_pass\models.frag -o ..\..\src\modes\playgame\render\passes\model_pass\models.frag.spv
glslangValidator.exe -V ..\..\src\modes\playgame\render\passes\model_pass\models.vert -o ..\..\src\modes\playgame\render\passes\model_pass\models.vert.spv
