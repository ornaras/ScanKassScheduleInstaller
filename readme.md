# Библиотека для установки SkatWorkerAPI

## Методы присутствующие в библиотеке

- **bool is_installed()** - возвращает `true` если SkatWorkerAPI найден в системе, иначе `false`
- **int install()** - скачивание и тихая установка SkatWorkerAPI с зависимостями; возвращает код ошибки
- **int adv_install(bool is_slient)** - вариация метода install с настройкой показа UI установщиков зависимостей

## Зависимости
- [Visual C++ 2015-2022 Redistributable (x86)](https://aka.ms/vs/17/release/vc_redist.x86.exe)

## Поддерживаемые ОС

- Windows 10 (x86/amd64)
- Windows 11 (amd64)

## Таблица ошибок

Код | Пояснение
--- | ---
0 | **Установка завершена успешно**
1 | Планировщик уже установлен
2 | Не удалось определить архитектуру системы
3 | Не удалось настроить планировщик

## Пример внедрения в .NET проект

```csharp
using System.Runtime.InteropServices;

public static class Program
{
    static void Main()
    {
        if (SkatWorker.IsInstalled())
            Console.WriteLine("Планировщик уже установлен");
        else {
            var code = SkatWorker.Install();
            if(code == 0)
                Console.WriteLine("Планировщик успешно установлен");
            else
                Console.WriteLine($"При установке планировщика произошла ошибка (Код: {code}).");
        }
    }  
}

internal static class SkatWorker
{
    private const string DllName = "skatworker_installation.dll";

    [DllImport(DllName, EntryPoint = "install")]
    public static extern int Install();
    
    [DllImport(DllName, EntryPoint = "is_installed")]
    public static extern bool IsInstalled();
}
```
