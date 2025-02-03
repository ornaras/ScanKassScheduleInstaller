# Библиотека для установки SkatWorkerAPI

## Процесс внедрения в .NET проект

1) Добавить DLL-библиотеку в проект и включить копирование в выходной каталог
2) В код добавить следующие строки:
    ```csharp
    // Для загрузки внешних библиотек, написанных на C/C++
    [DllImport("kernel32", SetLastError = true, CharSet = CharSet.Unicode)]
    public static extern nint LoadLibrary(string lpFileName);
    // Установка SkatWorkerAPI
    [DllImport("skatworker_installer.dll")]
    public static extern Task install();
    // Проверка на присутствие SkatWorkerAPI в системе
    [DllImport("skatworker_installer.dll")]
    public static extern bool is_installed();
    ```
3) Необходимо зарегистрировать библиотеку в коде: `LoadLibrary("skatworker_installer.dll");`

## Пример

```csharp
using System.Runtime.InteropServices;

public static class Program
{
    static void Main()
    {
        Native.LoadLibrary("skatworker_installer.dll");
        if (Native.is_installed())
            Console.WriteLine("installed");
        else
            Task.Run(Native.InstallSkatWorker);
        Console.WriteLine("finish");
    }
}

internal static class Native
{
    [DllImport("kernel32", SetLastError = true, CharSet = CharSet.Unicode)]
    public static extern nint LoadLibrary(string lpFileName);
    
    [DllImport("skatworker_installer.dll", EntryPoint = "install")]
    public static extern Task InstallSkatWorker();
    
    [DllImport("skatworker_installer.dll", EntryPoint = "is_installed")]
    public static extern bool IsInstalled();
}
```