# Библиотека для установки SkatWorkerAPI

## Методы присутствующие в библиотеке

- install - асинхронный метод, скачивающий и устанавливающий SkatWorkerAPI и его зависимости и возвращает код ошибки (0 - успешно установлено)
- is_installed - функция возвращает `true` если SkatWorkerAPI найден в системе, иначе `false`

## Подводные камни

- Поддерживается только Windows на архитектурах AMD64 и x86
- Установка всех зависимостей SkatWorkerAPI производится не в "тихом" режиме
- Библиотека собрана только для x86-приложений

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
